//! End-to-end tests: real `SyncPeer` clients (the engine's collaboration
//! peer) talking to the relay over live websockets.

use std::time::Duration;

use futures_util::{SinkExt, StreamExt};
use tokio::net::{TcpListener, TcpStream};
use tokio::task::JoinHandle;
use tokio_tungstenite::tungstenite::Message as WsMessage;
use tokio_tungstenite::{MaybeTlsStream, WebSocketStream};

use ironcalc_base::crdt::SyncPeer;
use ironcalc_base::UserModel;

use ironcalc_collab_server::server::{run, Rooms};

type Socket = WebSocketStream<MaybeTlsStream<TcpStream>>;

struct Client {
    um: UserModel<'static>,
    peer: SyncPeer,
    socket: Socket,
}

async fn start_server(rooms: Rooms) -> (String, JoinHandle<()>) {
    let listener = TcpListener::bind("127.0.0.1:0").await.expect("bind");
    let addr = listener.local_addr().expect("local addr");
    let handle = tokio::spawn(run(listener, rooms));
    (format!("ws://{addr}"), handle)
}

impl Client {
    /// Connects, runs the connection handshake and pumps until the line goes
    /// quiet — after this the client is converged with the room.
    async fn connect(url: &str, room: &str, client_id: u64) -> Client {
        let mut um = UserModel::new_empty("workbook", "en", "UTC", "en").expect("model");
        let peer = SyncPeer::attach(&mut um, client_id).expect("attach");
        let (socket, _) = tokio_tungstenite::connect_async(format!("{url}/{room}"))
            .await
            .expect("connect");
        let mut client = Client { um, peer, socket };
        for frame in client.peer.start_sync() {
            client.send(frame).await;
        }
        client.pump_quiet().await;
        client
    }

    async fn send(&mut self, frame: Vec<u8>) {
        self.socket
            .send(WsMessage::Binary(frame))
            .await
            .expect("send");
    }

    /// Translates local edits and ships them.
    async fn flush(&mut self) {
        if let Some(frame) = self.peer.flush_local(&mut self.um).expect("flush") {
            self.send(frame).await;
        }
    }

    /// Handles one incoming frame if one arrives within the timeout;
    /// returns whether one did.
    async fn step(&mut self, wait: Duration) -> bool {
        let incoming = tokio::time::timeout(wait, self.socket.next()).await;
        match incoming {
            Ok(Some(Ok(WsMessage::Binary(data)))) => {
                let outcome = self
                    .peer
                    .handle_frame(&mut self.um, &data)
                    .expect("handle frame");
                for reply in outcome.replies {
                    self.send(reply).await;
                }
                true
            }
            Ok(Some(Ok(_))) => true,
            Ok(Some(Err(e))) => panic!("socket error: {e}"),
            Ok(None) => false,
            Err(_) => false,
        }
    }

    /// Pumps frames until the line is quiet for a while.
    async fn pump_quiet(&mut self) {
        while self.step(Duration::from_millis(400)).await {}
    }

    /// Pumps frames until `pred` holds (fails the test after ~10s).
    async fn pump_until(&mut self, pred: impl Fn(&Client) -> bool, what: &str) {
        for _ in 0..100 {
            if pred(self) {
                return;
            }
            self.step(Duration::from_millis(100)).await;
        }
        panic!("timed out waiting for: {what}");
    }
}

fn cell(client: &Client, row: i32, column: i32) -> String {
    client
        .um
        .get_formatted_cell_value(0, row, column)
        .expect("cell value")
}

#[tokio::test(flavor = "multi_thread")]
async fn two_clients_converge_through_the_relay() {
    let (url, server) = start_server(Rooms::default()).await;
    let mut a = Client::connect(&url, "meeting", 1).await;
    let mut b = Client::connect(&url, "meeting", 2).await;

    // A's edit reaches B.
    a.um.set_user_input(0, 1, 1, "5").expect("edit");
    a.flush().await;
    b.pump_until(|c| cell(c, 1, 1) == "5", "A's edit on B").await;

    // B answers with a formula over it; A converges and evaluates.
    b.um.set_user_input(0, 1, 2, "=A1+1").expect("edit");
    b.flush().await;
    a.pump_until(|c| cell(c, 1, 2) == "6", "B's formula on A").await;

    // Rooms are isolated: a third client in another room sees none of it.
    let mut c = Client::connect(&url, "other-room", 3).await;
    assert_eq!(cell(&c, 1, 1), "");
    c.um.set_user_input(0, 9, 9, "999").expect("edit");
    c.flush().await;
    a.pump_quiet().await;
    assert_eq!(cell(&a, 9, 9), "");

    server.abort();
}

#[tokio::test(flavor = "multi_thread")]
async fn presence_relays_and_prunes_on_disconnect() {
    let (url, server) = start_server(Rooms::default()).await;
    let mut a = Client::connect(&url, "standup", 11).await;
    let mut b = Client::connect(&url, "standup", 12).await;

    let frame = a.peer.set_presence(r#"{"name":"ana"}"#).expect("presence");
    a.send(frame).await;
    b.pump_until(
        |c| c.peer.presence().iter().any(|(id, _)| *id == 11),
        "A's presence on B",
    )
    .await;

    // A late joiner learns the existing presence from the server's hello.
    let mut c = Client::connect(&url, "standup", 13).await;
    c.pump_until(
        |c| c.peer.presence().iter().any(|(id, _)| *id == 11),
        "A's presence on late joiner",
    )
    .await;

    // A drops without clearing its presence; the server prunes and tells B.
    drop(a);
    b.pump_until(
        |c| !c.peer.presence().iter().any(|(id, _)| *id == 11),
        "A's presence pruned on B",
    )
    .await;

    server.abort();
}

#[tokio::test(flavor = "multi_thread")]
async fn room_survives_server_restart() {
    let data_dir = std::env::temp_dir().join(format!(
        "ironcalc-collab-restart-test-{}",
        std::process::id()
    ));
    let _ = std::fs::remove_dir_all(&data_dir);

    let (url, server) = start_server(Rooms::new(Some(data_dir.clone()))).await;
    {
        let mut a = Client::connect(&url, "budget", 21).await;
        a.um.set_user_input(0, 1, 1, "42").expect("edit");
        a.um.set_user_input(0, 2, 1, "=A1*2").expect("edit");
        a.flush().await;
        // The server fans integrated updates back to their source too; the
        // append to the log happens before that broadcast, so seeing the
        // echo proves the edits are persisted.
        assert!(
            a.step(Duration::from_secs(5)).await,
            "no echo of the flushed update"
        );
        a.pump_quiet().await;
    }
    server.abort();
    let _ = tokio::time::timeout(Duration::from_secs(1), server).await;

    // A fresh server process over the same data dir: a new client finds the
    // workbook, including evaluated formulas.
    let (url, server) = start_server(Rooms::new(Some(data_dir.clone()))).await;
    let mut b = Client::connect(&url, "budget", 22).await;
    b.pump_until(|c| cell(c, 1, 1) == "42", "restored cell").await;
    assert_eq!(cell(&b, 2, 1), "84");

    server.abort();
    let _ = std::fs::remove_dir_all(&data_dir);
}
