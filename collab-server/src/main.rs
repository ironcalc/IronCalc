use tokio::net::TcpListener;

use ironcalc_collab_server::server::{run, Rooms};

#[tokio::main]
async fn main() {
    let addr = std::env::args()
        .nth(1)
        .unwrap_or_else(|| "127.0.0.1:9000".to_string());
    let data_dir = std::env::args().nth(2).map(std::path::PathBuf::from);
    let listener = TcpListener::bind(&addr).await.expect("cannot bind address");
    match &data_dir {
        Some(dir) => eprintln!(
            "ironcalc collab relay on ws://{addr}/<room>, persisting to {}",
            dir.display()
        ),
        None => eprintln!("ironcalc collab relay on ws://{addr}/<room> (ephemeral, no data dir)"),
    }
    run(listener, Rooms::new(data_dir)).await;
}
