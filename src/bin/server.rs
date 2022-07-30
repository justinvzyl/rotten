//! rotten server
//! This file is the entry point for the server implemented in the library.

use rotten::{server, DEFAULT_PORT};

use tokio::net::UdpSocket;
use tokio::signal;
use tracing::{info, Level};
use tracing_subscriber::FmtSubscriber;

#[tokio::main]
async fn main() -> rotten::Result<()> {
    // create a tracing subscriber and set as default
    let subscriber = FmtSubscriber::builder()
        .with_max_level(Level::TRACE)
        .finish();
    tracing::subscriber::set_global_default(subscriber).expect("setting default subscriber failed");

    // Bind a UDP listener
    let socket = UdpSocket::bind(&format!("127.0.0.1:{}", DEFAULT_PORT)).await?;

    // run the server
    info!("listening on 127.0.0.1:{}", DEFAULT_PORT);
    server::run(socket, signal::ctrl_c()).await;

    Ok(())
}
