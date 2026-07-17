//! A collaboration room: one shared yrs document, its awareness (presence)
//! state, and a broadcast channel fanning frames out to every connection.

use std::path::Path;
use std::sync::{Arc, Mutex};

use tokio::sync::broadcast;
use yrs::block::ClientID;
use yrs::sync::{Awareness, Message, MessageReader, SyncMessage};
use yrs::updates::decoder::{Decode, DecoderV1};
use yrs::updates::encoder::Encode;
use yrs::{Doc, ReadTxn, StateVector, Subscription, Transact, Update};

use crate::protocol::{classify_update, UpdateFit, EMPTY_UPDATE_V1};
use crate::storage::Storage;

/// Cap on gapped updates held for retry; beyond it the stash is dropped —
/// the resync request already sent re-delivers everything.
const MAX_STASHED_UPDATES: usize = 64;

/// Frames queued per connection before a slow client is disconnected (it
/// reconnects and heals through the handshake).
const BROADCAST_CAPACITY: usize = 1024;

pub struct Room {
    state: Mutex<RoomState>,
    tx: broadcast::Sender<Vec<u8>>,
}

struct RoomState {
    /// Owns the room document (`awareness.doc()`).
    awareness: Awareness,
    /// Updates that arrived with a causal gap, retried after every apply.
    stash: Vec<Vec<u8>>,
    /// Shared with the update observer, which appends every integrated
    /// update; compaction happens outside the observer (a transaction is
    /// still open while it runs).
    storage: Option<Arc<Mutex<Storage>>>,
    _update_sub: Subscription,
}

impl Room {
    /// Creates a room, replaying its persisted state when `data_dir` is set.
    /// (Replay happens before the update observer subscribes, so restored
    /// updates are neither re-logged nor broadcast.)
    pub fn new(data_dir: Option<&Path>, name: &str) -> Arc<Room> {
        let (tx, _) = broadcast::channel(BROADCAST_CAPACITY);
        let doc = Doc::new();
        let storage = data_dir.and_then(|dir| match Storage::open(dir, name) {
            Ok((storage, replay)) => {
                let mut txn = doc.transact_mut();
                for payload in replay {
                    let Ok(update) = Update::decode_v1(&payload) else {
                        eprintln!("room {name}: skipping undecodable persisted update");
                        continue;
                    };
                    if let Err(e) = txn.apply_update(update) {
                        eprintln!("room {name}: skipping unappliable persisted update: {e}");
                    }
                }
                Some(Arc::new(Mutex::new(storage)))
            }
            Err(e) => {
                eprintln!("room {name}: persistence disabled, cannot open storage: {e}");
                None
            }
        });
        let update_sub = {
            let tx = tx.clone();
            let storage = storage.clone();
            doc.observe_update_v1(move |_txn, event| {
                // Every update integrated into the room doc — no matter which
                // connection or handshake step carried it — is fanned out to
                // the whole room. Clients deduplicate (updates are
                // idempotent), so no source filtering is needed.
                if event.update != EMPTY_UPDATE_V1 {
                    if let Some(storage) = &storage {
                        if let Ok(mut storage) = storage.lock() {
                            if let Err(e) = storage.append(&event.update) {
                                eprintln!("collab room: cannot persist update: {e}");
                            }
                        }
                    }
                    let frame =
                        Message::Sync(SyncMessage::Update(event.update.clone())).encode_v1();
                    let _ = tx.send(frame);
                }
            })
            .expect("fresh doc accepts an update observer")
        };
        let awareness = Awareness::new(doc);
        Arc::new(Room {
            state: Mutex::new(RoomState {
                awareness,
                stash: Vec::new(),
                storage,
                _update_sub: update_sub,
            }),
            tx,
        })
    }

    pub fn subscribe(&self) -> broadcast::Receiver<Vec<u8>> {
        self.tx.subscribe()
    }

    /// Frames sent to a client right after it connects: the server's state
    /// vector (the client answers with what the server misses) and the
    /// currently known presence states.
    pub fn hello(&self) -> Vec<Vec<u8>> {
        let state = self.state.lock().expect("room lock");
        let sv = state.awareness.doc().transact().state_vector();
        let mut frames = vec![Message::Sync(SyncMessage::SyncStep1(sv)).encode_v1()];
        if let Ok(update) = state.awareness.update() {
            if !update.clients.is_empty() {
                frames.push(Message::Awareness(update).encode_v1());
            }
        }
        frames
    }

    /// Handles one frame from a connection and returns the direct replies.
    /// Applied updates are broadcast via the document observer; awareness
    /// updates are broadcast explicitly. `presence_clients` accumulates the
    /// awareness client ids this connection has announced, for pruning on
    /// disconnect.
    pub fn handle_frame(
        &self,
        frame: &[u8],
        presence_clients: &mut Vec<u64>,
    ) -> Result<Vec<Vec<u8>>, String> {
        let mut decoder = DecoderV1::from(frame);
        let messages: Vec<Message> = MessageReader::new(&mut decoder)
            .collect::<Result<_, _>>()
            .map_err(|e| format!("bad frame: {e}"))?;
        let mut replies = Vec::new();
        let mut resync_requested = false;
        let mut state = self.state.lock().expect("room lock");
        for message in messages {
            match message {
                Message::Sync(SyncMessage::SyncStep1(sv)) => {
                    let diff = state
                        .awareness
                        .doc()
                        .transact()
                        .encode_state_as_update_v1(&sv);
                    replies.push(Message::Sync(SyncMessage::SyncStep2(diff)).encode_v1());
                }
                Message::Sync(SyncMessage::SyncStep2(update))
                | Message::Sync(SyncMessage::Update(update)) => {
                    if update == EMPTY_UPDATE_V1 {
                        continue;
                    }
                    let decoded =
                        Update::decode_v1(&update).map_err(|e| format!("bad update: {e}"))?;
                    let local = state.awareness.doc().transact().state_vector();
                    match classify_update(&decoded, &local) {
                        UpdateFit::Empty => {}
                        UpdateFit::Gap => {
                            if state.stash.len() >= MAX_STASHED_UPDATES {
                                state.stash.clear();
                            }
                            state.stash.push(update);
                            if !resync_requested {
                                resync_requested = true;
                                replies
                                    .push(Message::Sync(SyncMessage::SyncStep1(local)).encode_v1());
                            }
                        }
                        UpdateFit::Content => {
                            apply_update(&mut state, decoded)?;
                            retry_stash(&mut state)?;
                        }
                    }
                }
                Message::AwarenessQuery => {
                    if let Ok(update) = state.awareness.update() {
                        if !update.clients.is_empty() {
                            replies.push(Message::Awareness(update).encode_v1());
                        }
                    }
                }
                Message::Awareness(update) => {
                    for client in update.clients.keys() {
                        let id = client.get();
                        if !presence_clients.contains(&id) {
                            presence_clients.push(id);
                        }
                    }
                    let frame = Message::Awareness(update.clone()).encode_v1();
                    state
                        .awareness
                        .apply_update(update)
                        .map_err(|e| format!("bad awareness update: {e}"))?;
                    let _ = self.tx.send(frame);
                }
                Message::Auth(_) | Message::Custom(..) => {}
            }
        }
        maybe_compact(&state);
        Ok(replies)
    }

    /// Prunes the presence of a disconnected client's ids, broadcasts the
    /// removal to the room, and takes the chance to compact the log.
    pub fn disconnect(&self, presence_clients: &[u64]) {
        let mut state = self.state.lock().expect("room lock");
        for &id in presence_clients {
            state.awareness.remove_state(ClientID::new(id));
        }
        if !presence_clients.is_empty() {
            let ids = presence_clients.iter().map(|&id| ClientID::new(id));
            if let Ok(update) = state.awareness.update_with_clients(ids) {
                let _ = self.tx.send(Message::Awareness(update).encode_v1());
            }
        }
        maybe_compact(&state);
    }
}

/// Compacts the log into a fresh snapshot when it has outgrown its limit.
/// Never called from the update observer: the observer runs while the
/// transaction that triggered it is still open, so reading the full state
/// there would deadlock.
fn maybe_compact(state: &RoomState) {
    let Some(storage) = &state.storage else {
        return;
    };
    let Ok(mut storage) = storage.lock() else {
        return;
    };
    if !storage.needs_compaction() {
        return;
    }
    let full_state = state
        .awareness
        .doc()
        .transact()
        .encode_state_as_update_v1(&StateVector::default());
    if let Err(e) = storage.compact(&full_state) {
        eprintln!("collab room: compaction failed: {e}");
    }
}

fn apply_update(state: &mut RoomState, update: Update) -> Result<(), String> {
    let mut txn = state.awareness.doc().transact_mut();
    txn.apply_update(update).map_err(|e| format!("apply: {e}"))
}

fn retry_stash(state: &mut RoomState) -> Result<(), String> {
    loop {
        let local = state.awareness.doc().transact().state_vector();
        let mut ready = None;
        let mut index = 0;
        while index < state.stash.len() {
            let update = Update::decode_v1(&state.stash[index])
                .map_err(|e| format!("bad stashed update: {e}"))?;
            match classify_update(&update, &local) {
                UpdateFit::Empty => {
                    state.stash.swap_remove(index);
                }
                UpdateFit::Content => {
                    state.stash.swap_remove(index);
                    ready = Some(update);
                    break;
                }
                UpdateFit::Gap => index += 1,
            }
        }
        match ready {
            Some(update) => apply_update(state, update)?,
            None => return Ok(()),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use yrs::{GetString, Map, Text};

    fn frame_with_update(update: Vec<u8>) -> Vec<u8> {
        Message::Sync(SyncMessage::Update(update)).encode_v1()
    }

    /// Extracts the update carried by the first SyncStep2 reply.
    fn sync_step2(replies: &[Vec<u8>]) -> Vec<u8> {
        for reply in replies {
            let mut decoder = DecoderV1::from(reply.as_slice());
            for message in MessageReader::new(&mut decoder) {
                if let Ok(Message::Sync(SyncMessage::SyncStep2(update))) = message {
                    return update;
                }
            }
        }
        panic!("no SyncStep2 in replies");
    }

    #[test]
    fn room_state_survives_recreation() {
        let dir = std::env::temp_dir().join(format!(
            "ironcalc-collab-room-test-{}",
            std::process::id()
        ));
        let _ = std::fs::remove_dir_all(&dir);
        let mut presence = Vec::new();

        // A "client" writes a value into the room.
        let client = Doc::with_client_id(7);
        let text = client.get_or_insert_text("t");
        text.insert(&mut client.transact_mut(), 0, "persisted");
        let update = client
            .transact()
            .encode_state_as_update_v1(&StateVector::default());
        {
            let room = Room::new(Some(&dir), "sheet");
            room.handle_frame(&frame_with_update(update), &mut presence)
                .unwrap();
        }

        // A fresh room instance (server restart) must still have it.
        let room = Room::new(Some(&dir), "sheet");
        let hello = Message::Sync(SyncMessage::SyncStep1(StateVector::default())).encode_v1();
        let replies = room.handle_frame(&hello, &mut presence).unwrap();
        let restored = Doc::new();
        let text = restored.get_or_insert_text("t");
        restored
            .transact_mut()
            .apply_update(Update::decode_v1(&sync_step2(&replies)).unwrap())
            .unwrap();
        assert_eq!(text.get_string(&restored.transact()), "persisted");
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn ephemeral_room_accepts_updates_without_data_dir() {
        let mut presence = Vec::new();
        let client = Doc::with_client_id(9);
        let map = client.get_or_insert_map("m");
        map.insert(&mut client.transact_mut(), "k", "v");
        let update = client
            .transact()
            .encode_state_as_update_v1(&StateVector::default());
        let room = Room::new(None, "scratch");
        room.handle_frame(&frame_with_update(update), &mut presence)
            .unwrap();
        let hello = Message::Sync(SyncMessage::SyncStep1(StateVector::default())).encode_v1();
        let replies = room.handle_frame(&hello, &mut presence).unwrap();
        assert!(!sync_step2(&replies).is_empty());
    }
}
