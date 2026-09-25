use std::time::Duration;

use futures_util::{SinkExt, StreamExt};
use ironcalc_base::crdt::SyncPeer;
use ironcalc_base::UserModel;
use ironcalc_collab_server::compress::unwrap_frame;
use tokio::net::TcpStream;
use tokio_tungstenite::tungstenite::Message as WsMessage;
use tokio_tungstenite::{MaybeTlsStream, WebSocketStream};

type Socket = WebSocketStream<MaybeTlsStream<TcpStream>>;

/// One collaborator: a workbook, its CRDT peer and a websocket to the relay.
struct Client {
    model: UserModel<'static>,
    peer: SyncPeer,
    socket: Socket,
}

impl Client {
    async fn connect(room_url: &str, client_id: u64) -> Client {
        let mut model = UserModel::new_empty("wb", "en", "UTC", "en").unwrap();
        let peer = SyncPeer::attach(&mut model, client_id).unwrap();
        let (socket, _) = tokio_tungstenite::connect_async(room_url).await.unwrap();
        let mut client = Client {
            model,
            peer,
            socket,
        };
        // Handshake: send our state vector, the relay answers with what we
        // miss and asks for what it misses. Pump until the line goes quiet.
        for frame in client.peer.start_sync() {
            client.send(frame).await;
        }
        while client.step(Duration::from_millis(300)).await {}
        client
    }

    async fn send(&mut self, frame: Vec<u8>) {
        self.socket.send(WsMessage::Binary(frame)).await.unwrap();
    }

    /// Ships pending local edits, if any.
    async fn flush(&mut self) {
        if let Some(frame) = self.peer.flush_local(&mut self.model).unwrap() {
            self.send(frame).await;
        }
    }

    /// Handles one incoming frame (answering its replies); false on timeout.
    async fn step(&mut self, wait: Duration) -> bool {
        match tokio::time::timeout(wait, self.socket.next()).await {
            Ok(Some(Ok(WsMessage::Binary(data)))) => {
                // Large relay frames arrive gzip-wrapped.
                let data = unwrap_frame(&data, 1 << 30).unwrap();
                let outcome = self.peer.handle_frame(&mut self.model, &data).unwrap();
                for reply in outcome.replies {
                    self.send(reply).await;
                }
                true
            }
            Ok(Some(Ok(_))) => true, // ping/pong/text: ignore
            _ => false,
        }
    }

    fn cell(&self, row: i32, column: i32) -> String {
        self.model.get_formatted_cell_value(0, row, column).unwrap()
    }
}

#[tokio::main]
async fn main() {
    let room = "ws://127.0.0.1:9000/demo";
    let mut alice = Client::connect(room, 1).await;
    let mut bob = Client::connect(room, 2).await;

    // Alice types into A1; Bob answers with a formula over it.
    alice.model.set_user_input(0, 1, 1, "5").unwrap();
    alice.flush().await;
    while bob.cell(1, 1) != "5" {
        assert!(
            bob.step(Duration::from_secs(5)).await,
            "no update from Alice"
        );
    }
    bob.model.set_user_input(0, 1, 2, "=A1+1").unwrap();
    bob.flush().await;
    while alice.cell(1, 2) != "6" {
        assert!(
            alice.step(Duration::from_secs(5)).await,
            "no update from Bob"
        );
    }
    println!("Alice sees B1 = {}", alice.cell(1, 2));
}
