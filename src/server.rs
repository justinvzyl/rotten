use std::future::Future;
use std::sync::Arc;
use std::time::Duration;
use tokio::net::UdpSocket;
use tokio::sync::{broadcast, mpsc, Semaphore};
use tokio::time::sleep;
use tracing::{debug, error, info};

///  Run the rotten server.
///
/// Accepts UDP connections from the provided listener. For each inbound
/// connection, a task is spawned to handle that connection. The server
/// runs until the `shutdown` future completes, after which the server
/// will shut down gracefully
///
/// `tokio::signal::ctrl_c()` may be used as the `shutdown` argument.
pub async fn run(listener: UdpSocket, shutdown: impl Future) {
    // When the `shutdown` future completes, we must send a shutdown
    // message to all active connections. We use a broadcast channel
    // for this purpose. The receiver of the broadcast pair is ignored
    // and when one is required the subscribe() method on the send may
    // be used.
    let (notify_shutdown, _) = broadcast::channel(1);
    let (shutdown_complete_tx, shutdown_complete_rx) = mpsc::channel(1);

    // Init the listener state
    let mut server = Listener {
        listener,
        // db_holder: DbDropGuard::new(),
        limit_connections: Arc::new(Semaphore::new(256)),
        notify_shutdown,
        shutdown_complete_tx,
        shutdown_complete_rx,
    };

    // Concurrently run the server and listen for the `shutdown` signal. The
    // server task runs until an error occurs, therefore, under normal circumstances,
    // this `select!` runs until the `shutdown` signal is received.
    tokio::select! {
        res = server.run() => {
            // If an error is received here: CONDITIONS
            // then the server is shutting down
            //
            // Note: errors encountered when handling individual connections do not
            // bubble up to this point.
            if let Err(err) = res {
                error!(cause = %err, "failed to accept connections");
            }
        },
        _ = shutdown => {
            // The shutdown signal has been received.
            info!("shutting down server");
        }
    }
}

#[derive(Debug)]
struct Listener {
    listener: UdpSocket,
    // db_holder: DropGuard,
    limit_connections: Arc<Semaphore>,
    notify_shutdown: broadcast::Sender<()>,
    shutdown_complete_tx: mpsc::Sender<()>,
    shutdown_complete_rx: mpsc::Receiver<()>,
}

impl Listener {
    async fn run(&self) -> crate::Result<()> {
        loop {
            debug!("sleeping");
            sleep(Duration::from_secs(1)).await;
        }
    }
}
