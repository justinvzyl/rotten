//! rotten server
//! This file is the entry point for the server implemented in the library.

use rotten::{server, DEFAULT_PORT};

use tokio::net::UdpSocket;
use tokio::signal;

#[tokio::main]
async fn main() -> rotten::Result<()> {
    // Bind a UDP listener
    let listener = UdpSocket::bind(&format!("127.0.0.1:{}", DEFAULT_PORT)).await?;

    // run the server
    println!("Listening on 127.0.0.1:{}", DEFAULT_PORT);
    server::run(listener, signal::ctrl_c()).await;

    Ok(())
}
