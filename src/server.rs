use std::future::Future;
use tokio::net::UdpSocket;

pub async fn run(listener: UdpSocket, shutdown: impl Future) {}
