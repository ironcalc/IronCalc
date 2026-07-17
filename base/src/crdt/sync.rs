//! Y-sync protocol peer: frames a [`CollabSession`] for a real transport.
//!
//! The peer is transport-agnostic — the caller owns the pipe (websocket,
//! in-process channel, …) and shuttles opaque byte frames. Every frame is one
//! or more y-sync protocol messages ([`yrs::sync::Message`], lib0 v1
//! encoding), so the wire format is compatible with the standard y-websocket
//! framing. Document sync follows the y-sync handshake (state vectors +
//! diffs); presence rides the ephemeral y-awareness channel and never touches
//! the document.
//!
//! Topology note: the session keeps a single "sent" state vector, so a peer
//! assumes one pipe to the rest of the room (a relay server, or one direct
//! peer). Multi-pipe meshes converge too — the handshake heals any gap on
//! (re)connect — they just may re-send updates a given peer already has.

use std::sync::{Arc, Mutex};

use yrs::sync::{Awareness, Message, MessageReader, SyncMessage};
use yrs::updates::decoder::{Decode, DecoderV1};
use yrs::updates::encoder::Encode;
use yrs::{Doc, Origin, StateVector, Subscription, Update};

use super::session::REMOTE_ORIGIN;
use super::CollabSession;
use crate::UserModel;

/// An empty v1 update: zero structs, empty delete set. Guarded by a test.
const EMPTY_UPDATE_V1: &[u8] = &[0, 0];

/// Cap on updates held back because of a causal gap; beyond this the stash is
/// dropped — the resync handshake already requested re-delivers everything.
const MAX_STASHED_UPDATES: usize = 64;

/// How an incoming document update was delivered.
enum Delivery {
    /// Applied to the document (possibly unblocking stashed updates).
    Applied,
    /// Contained nothing new — duplicate or already-covered delivery.
    Known,
    /// Held back because of a causal gap; a resync request should go out.
    Stashed,
}

/// What handling an incoming frame produced.
#[derive(Default)]
pub struct FrameOutcome {
    /// Frames to send back over the same pipe.
    pub replies: Vec<Vec<u8>>,
    /// A document update was applied (the model changed — re-render).
    pub applied_update: bool,
    /// The presence map changed (re-render cursors).
    pub presence_changed: bool,
}

/// A collaboration peer: one [`CollabSession`] plus the awareness (presence)
/// state, speaking the y-sync protocol in byte frames.
pub struct SyncPeer {
    session: CollabSession,
    awareness: Awareness,
    /// Updates that arrived with a causal gap (out-of-order delivery across a
    /// reconnect), held back and retried after every successful apply. yrs
    /// must never see a gapped update: its pending queue silently fails to
    /// re-integrate parked map-overwrite items once the gap fills (yrs
    /// 0.27.3), which would desync the keep-sets.
    stash: Vec<Vec<u8>>,
    /// Incremental updates from local transactions (fed by the document's
    /// update observer, remote-origin transactions filtered out), drained by
    /// [`SyncPeer::flush_local`]. Incremental payloads carry exactly the new
    /// blocks and deletions — unlike `encode_state_as_update`, which always
    /// re-ships the full delete set.
    outbox: Arc<Mutex<Vec<Vec<u8>>>>,
    _update_sub: Subscription,
}

/// How an update relates to a document with state `local`.
enum UpdateFit {
    /// Some client's blocks start beyond the locally known clock: applying
    /// would leave a causal gap.
    Gap,
    /// Causally contiguous and carries new blocks and/or deletions.
    Content,
    /// Everything in it is already known.
    Empty,
}

fn classify_update(update: &Update, local: &StateVector) -> UpdateFit {
    let mut novel = false;
    let insertions = update.insertions(true);
    for client in insertions.client_ids() {
        let Some(id_range) = insertions.get(&client) else {
            continue;
        };
        let mut ranges: Vec<(u32, u32)> = id_range.iter().map(|r| (r.0.start, r.0.end)).collect();
        ranges.sort_unstable();
        let mut cursor = local.get(&client);
        for (start, end) in ranges {
            if end <= cursor {
                continue;
            }
            if start > cursor {
                return UpdateFit::Gap;
            }
            novel = true;
            cursor = end;
        }
    }
    if novel || !update.delete_set().is_empty() {
        UpdateFit::Content
    } else {
        UpdateFit::Empty
    }
}

#[cfg(not(all(target_family = "wasm", target_os = "unknown")))]
fn new_awareness(doc: Doc) -> Awareness {
    Awareness::new(doc)
}

#[cfg(all(target_family = "wasm", target_os = "unknown"))]
fn new_awareness(doc: Doc) -> Awareness {
    // `SystemTime` is unavailable on wasm32-unknown-unknown. Awareness
    // timestamps only feed idle-pruning heuristics we do not use (the relay
    // prunes on disconnect), so a constant clock is fine.
    Awareness::with_clock(doc, || 0u64)
}

impl SyncPeer {
    /// Attaches a peer to a model, bootstrapping the document from the
    /// current workbook state (see [`CollabSession::attach`]).
    pub fn attach(um: &mut UserModel, client_id: u64) -> Result<SyncPeer, String> {
        let session = CollabSession::attach(um, client_id)?;
        let awareness = new_awareness(session.doc_handle());
        let outbox: Arc<Mutex<Vec<Vec<u8>>>> = Arc::new(Mutex::new(Vec::new()));
        let update_sub = {
            let outbox = Arc::clone(&outbox);
            let remote: Origin = REMOTE_ORIGIN.into();
            session
                .doc_handle()
                .observe_update_v1(move |txn, event| {
                    if txn.origin() != Some(&remote) && event.update != EMPTY_UPDATE_V1 {
                        if let Ok(mut queued) = outbox.lock() {
                            queued.push(event.update.clone());
                        }
                    }
                })
                .map_err(|e| format!("collab: cannot observe updates: {e}"))?
        };
        Ok(SyncPeer {
            session,
            awareness,
            stash: Vec::new(),
            outbox,
            _update_sub: update_sub,
        })
    }

    /// The underlying session (test hooks, direct update access).
    pub fn session(&self) -> &CollabSession {
        &self.session
    }

    pub fn client_id(&self) -> u64 {
        self.awareness.client_id().get()
    }

    /// Frames to send when a connection (re)opens: our state vector (their
    /// reply is everything we miss), a presence query, and our own presence
    /// if one is set. Receiving the state vector makes the other side answer
    /// with a diff *and* our handler answers their `SyncStep1` symmetrically,
    /// so running `start_sync` on both ends completes the handshake.
    pub fn start_sync(&self) -> Vec<Vec<u8>> {
        let sv = self.session.state_vector_raw();
        let mut frames = vec![
            Message::Sync(SyncMessage::SyncStep1(sv)).encode_v1(),
            Message::AwarenessQuery.encode_v1(),
        ];
        if let Ok(update) = self.awareness.update_with_clients([self.awareness.client_id()]) {
            frames.push(Message::Awareness(update).encode_v1());
        }
        frames
    }

    /// Handles one incoming frame (which may pack several protocol messages)
    /// and returns the frames to send back plus what changed locally.
    pub fn handle_frame(
        &mut self,
        um: &mut UserModel,
        frame: &[u8],
    ) -> Result<FrameOutcome, String> {
        let mut outcome = FrameOutcome::default();
        let mut resync_requested = false;
        let mut decoder = DecoderV1::from(frame);
        // Collect first: decoding borrows the frame, handling borrows self.
        let messages: Vec<Message> = MessageReader::new(&mut decoder)
            .collect::<Result<_, _>>()
            .map_err(|e| format!("collab: bad frame: {e}"))?;
        for message in messages {
            match message {
                Message::Sync(SyncMessage::SyncStep1(sv)) => {
                    // Answer with everything they are missing, including our
                    // not-yet-flushed local edits.
                    self.session.translate_queue(um)?;
                    let diff = self.session.handshake_diff(&sv);
                    outcome
                        .replies
                        .push(Message::Sync(SyncMessage::SyncStep2(diff)).encode_v1());
                }
                Message::Sync(SyncMessage::SyncStep2(update))
                | Message::Sync(SyncMessage::Update(update)) => {
                    if update != EMPTY_UPDATE_V1 {
                        match self.apply_or_stash(um, &update)? {
                            Delivery::Applied => outcome.applied_update = true,
                            Delivery::Known => {}
                            Delivery::Stashed => {
                                if !resync_requested {
                                    // Gapped update: ask the far side for a
                                    // proper diff so the hole fills even if
                                    // the missing update never arrives on
                                    // its own.
                                    resync_requested = true;
                                    let sv = self.session.state_vector_raw();
                                    outcome.replies.push(
                                        Message::Sync(SyncMessage::SyncStep1(sv)).encode_v1(),
                                    );
                                }
                            }
                        }
                    }
                }
                Message::AwarenessQuery => {
                    let update = self
                        .awareness
                        .update()
                        .map_err(|e| format!("collab: awareness: {e}"))?;
                    if !update.clients.is_empty() {
                        outcome.replies.push(Message::Awareness(update).encode_v1());
                    }
                }
                Message::Awareness(update) => {
                    let summary = self
                        .awareness
                        .apply_update_summary(update)
                        .map_err(|e| format!("collab: awareness: {e}"))?;
                    if let Some(summary) = summary {
                        outcome.presence_changed |= !summary.added.is_empty()
                            || !summary.updated.is_empty()
                            || !summary.removed.is_empty();
                    }
                }
                Message::Auth(_) | Message::Custom(..) => {}
            }
        }
        Ok(outcome)
    }

    /// Applies an update if it is causally contiguous with the local state;
    /// stashes it for retry otherwise. Applying may unblock stashed updates,
    /// which are then drained to a fixpoint.
    fn apply_or_stash(&mut self, um: &mut UserModel, bytes: &[u8]) -> Result<Delivery, String> {
        let update = Update::decode_v1(bytes).map_err(|e| format!("collab: bad update: {e}"))?;
        let local = self.session.state_vector_raw();
        match classify_update(&update, &local) {
            UpdateFit::Empty => Ok(Delivery::Known),
            UpdateFit::Gap => {
                if self.stash.len() >= MAX_STASHED_UPDATES {
                    // The resync handshake re-delivers everything anyway.
                    self.stash.clear();
                }
                self.stash.push(bytes.to_vec());
                Ok(Delivery::Stashed)
            }
            UpdateFit::Content => {
                self.session.apply_remote(um, bytes)?;
                self.retry_stash(um)?;
                Ok(Delivery::Applied)
            }
        }
    }

    /// Re-examines stashed updates until no more of them can be applied;
    /// drops the ones the document state has meanwhile made redundant.
    fn retry_stash(&mut self, um: &mut UserModel) -> Result<(), String> {
        loop {
            let local = self.session.state_vector_raw();
            let mut ready = None;
            let mut index = 0;
            while index < self.stash.len() {
                let update = Update::decode_v1(&self.stash[index])
                    .map_err(|e| format!("collab: bad stashed update: {e}"))?;
                match classify_update(&update, &local) {
                    UpdateFit::Empty => {
                        self.stash.swap_remove(index);
                    }
                    UpdateFit::Content => {
                        ready = Some(self.stash.swap_remove(index));
                        break;
                    }
                    UpdateFit::Gap => index += 1,
                }
            }
            match ready {
                Some(bytes) => self.session.apply_remote(um, &bytes)?,
                None => return Ok(()),
            }
        }
    }

    /// Translates pending local edits and returns the update frame to
    /// broadcast, or `None` when there is nothing new to send. The frame
    /// carries the incremental updates of the local transactions since the
    /// last flush, merged into one.
    pub fn flush_local(&mut self, um: &mut UserModel) -> Result<Option<Vec<u8>>, String> {
        self.session.translate_queue(um)?;
        let payloads: Vec<Vec<u8>> = {
            let mut queued = self
                .outbox
                .lock()
                .map_err(|_| "collab: outbox poisoned".to_string())?;
            std::mem::take(&mut *queued)
        };
        let update = match payloads.len() {
            0 => return Ok(None),
            1 => payloads.into_iter().next().expect("len checked"),
            _ => {
                let mut updates = Vec::with_capacity(payloads.len());
                for payload in &payloads {
                    updates.push(
                        Update::decode_v1(payload)
                            .map_err(|e| format!("collab: bad outbox update: {e}"))?,
                    );
                }
                Update::merge_updates(updates).encode_v1()
            }
        };
        Ok(Some(
            Message::Sync(SyncMessage::Update(update)).encode_v1(),
        ))
    }

    /// Sets this client's presence state (an opaque JSON string — the caller
    /// owns the schema: user name, selection, …) and returns the frame to
    /// broadcast.
    pub fn set_presence(&mut self, json: &str) -> Result<Vec<u8>, String> {
        self.awareness.set_local_state_raw(json);
        self.local_presence_frame()
    }

    /// Clears this client's presence (send before disconnecting) and returns
    /// the frame to broadcast.
    pub fn clear_presence(&mut self) -> Result<Vec<u8>, String> {
        self.awareness.clean_local_state();
        self.local_presence_frame()
    }

    fn local_presence_frame(&self) -> Result<Vec<u8>, String> {
        let update = self
            .awareness
            .update_with_clients([self.awareness.client_id()])
            .map_err(|e| format!("collab: awareness: {e}"))?;
        Ok(Message::Awareness(update).encode_v1())
    }

    /// The current presence map: `(client_id, state-json)` for every client
    /// with a live state, including this one.
    pub fn presence(&self) -> Vec<(u64, String)> {
        let mut states: Vec<(u64, String)> = self
            .awareness
            .iter()
            .filter_map(|(id, state)| state.data.map(|data| (id.get(), data.to_string())))
            .collect();
        states.sort();
        states
    }
}
