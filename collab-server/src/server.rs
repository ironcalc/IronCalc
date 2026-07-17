//! Accept loop and per-connection pump.

use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};

use futures_util::{SinkExt, StreamExt};
use tokio::net::{TcpListener, TcpStream};
use tokio::sync::broadcast;
use tokio_tungstenite::tungstenite::handshake::server::{Request, Response};
use tokio_tungstenite::tungstenite::Message as WsMessage;

use crate::room::Room;

/// The server's room registry. Rooms live for the lifetime of the process;
/// with a data directory they also survive restarts (snapshot + update log).
#[derive(Default, Clone)]
pub struct Rooms {
    rooms: Arc<Mutex<HashMap<String, Arc<Room>>>>,
    data_dir: Option<PathBuf>,
}

impl Rooms {
    /// A registry persisting each room under `data_dir` (`None` = ephemeral).
    pub fn new(data_dir: Option<PathBuf>) -> Rooms {
        Rooms {
            rooms: Arc::default(),
            data_dir,
        }
    }

    pub fn get_or_create(&self, name: &str) -> Arc<Room> {
        let mut rooms = self.rooms.lock().expect("rooms lock");
        Arc::clone(
            rooms
                .entry(name.to_string())
                .or_insert_with(|| Room::new(self.data_dir.as_deref(), name)),
        )
    }
}

/// Room names come from URL paths and double as file names — keep them to a
/// character set that cannot traverse the data directory.
fn valid_room_name(name: &str) -> bool {
    !name.is_empty()
        && name.len() <= 64
        && !name.starts_with('.')
        && name
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || matches!(c, '-' | '_' | '.'))
}

/// Serves connections on `listener` until the listener fails. The room a
/// client joins is the URL path of its websocket request.
pub async fn run(listener: TcpListener, rooms: Rooms) {
    while let Ok((stream, _)) = listener.accept().await {
        let rooms = rooms.clone();
        tokio::spawn(async move {
            let _ = handle_connection(stream, rooms).await;
        });
    }
}

// result_large_err: the Err type of tungstenite's header callback is fixed
// by its signature.
#[allow(clippy::result_large_err)]
async fn handle_connection(
    stream: TcpStream,
    rooms: Rooms,
) -> Result<(), tokio_tungstenite::tungstenite::Error> {
    let mut room_name = String::new();
    let websocket = tokio_tungstenite::accept_hdr_async(stream, |req: &Request, resp: Response| {
        room_name = req.uri().path().trim_matches('/').to_string();
        Ok(resp)
    })
    .await?;
    if room_name.is_empty() {
        room_name = "default".to_string();
    }
    if !valid_room_name(&room_name) {
        return Ok(());
    }
    let room = rooms.get_or_create(&room_name);
    let mut relay = room.subscribe();
    let (mut sink, mut source) = websocket.split();
    for frame in room.hello() {
        sink.send(WsMessage::Binary(frame)).await?;
    }
    // Awareness client ids this connection announced; pruned on disconnect.
    let mut presence_clients: Vec<u64> = Vec::new();
    let result = loop {
        tokio::select! {
            incoming = source.next() => match incoming {
                Some(Ok(WsMessage::Binary(data))) => {
                    match room.handle_frame(&data, &mut presence_clients) {
                        Ok(replies) => {
                            for reply in replies {
                                sink.send(WsMessage::Binary(reply)).await?;
                            }
                        }
                        // Protocol garbage: drop the client, keep the room.
                        Err(_) => break Ok(()),
                    }
                }
                Some(Ok(WsMessage::Close(_))) | None => break Ok(()),
                Some(Ok(_)) => {} // text/ping/pong: nothing to do
                Some(Err(e)) => break Err(e),
            },
            relayed = relay.recv() => match relayed {
                Ok(frame) => sink.send(WsMessage::Binary(frame)).await?,
                // Slow consumer lost frames: close; the client reconnects
                // and heals through the handshake.
                Err(broadcast::error::RecvError::Lagged(_)) => break Ok(()),
                Err(broadcast::error::RecvError::Closed) => break Ok(()),
            },
        }
    };
    room.disconnect(&presence_clients);
    result
}

#[cfg(test)]
mod tests {
    use super::valid_room_name;

    #[test]
    fn room_names_cannot_escape_the_data_dir() {
        assert!(valid_room_name("budget-2026_v1.ic"));
        assert!(valid_room_name("default"));
        assert!(!valid_room_name(""));
        assert!(!valid_room_name(".hidden"));
        assert!(!valid_room_name("a/b"));
        assert!(!valid_room_name("../escape"));
        assert!(!valid_room_name("with space"));
        assert!(!valid_room_name(&"x".repeat(65)));
    }
}
