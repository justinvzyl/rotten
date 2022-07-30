use crate::parser::{Frame, Parser};
use bytes::{BufMut, BytesMut};
use rand::random;
use std::collections::HashMap;
use std::future::Future;
use std::io;
use std::net::{IpAddr, SocketAddr};
use std::sync::{Arc, Mutex};
use tokio::net::UdpSocket;
use tracing::{debug, error, info};

type Db = Arc<Mutex<HashMap<String, Vec<Peer>>>>;

///  Run the rotten server.
///
/// Accepts UDP `connections` from the provided listener. For each inbound
/// dgram, a task is spawned to handle that dgram. The server
/// runs until the `shutdown` future completes, after which the server
/// will shut down gracefully
///
/// `tokio::signal::ctrl_c()` may be used as the `shutdown` argument.
pub async fn run(socket: UdpSocket, shutdown: impl Future) {
    // init the database
    let db: Db = Arc::new(Mutex::new(HashMap::new()));

    let db1 = Arc::clone(&db);

    // Init the listener state
    let server = Listener {
        listener: Arc::new(socket),
        db: db1,
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
    listener: Arc<UdpSocket>,
    db: Db,
}

impl Listener {
    /// Run the server
    ///
    /// Listen for inbound connections. For each inbound connection, spawn a
    /// task to process that connection.
    ///
    /// # Errors
    ///
    /// Returns `Err` if accepting returns an error.
    ///
    async fn run(&self) -> crate::Result<()> {
        info!("accepting inbound connections");
        loop {
            self.listener.readable().await?;

            // create buffer
            let mut buf = BytesMut::with_capacity(1024);

            match self.listener.try_recv_buf_from(&mut buf) {
                Ok((len, addr)) => {
                    debug!("new connection from {} with {} bytes of data", addr, len);
                    // init a handler for the dgram
                    let mut handler = Handler {
                        db: Arc::clone(&self.db),
                        from_addr: addr,
                        dgram_len: len,
                        connection: self.listener.clone(),
                        parser: Parser::new(buf),
                    };

                    // spawn a worker, and start up handler
                    tokio::spawn(async move {
                        // Process the data
                        handler.run().await.unwrap();
                    });
                }
                Err(ref e) if e.kind() == io::ErrorKind::WouldBlock => {
                    continue;
                }
                Err(e) => {
                    panic!("something bad")
                }
            }
        }
    }
}

struct Handler {
    db: Db,

    /// The address where the connection originated from
    from_addr: std::net::SocketAddr,

    /// The length of the datagram
    dgram_len: usize,

    /// A handle to the socket
    connection: Arc<UdpSocket>,

    /// A parser for the dgram
    parser: Parser,
}

impl Handler {
    async fn run(&mut self) -> crate::Result<()> {
        let buf = match self.parser.parse().await? {
            Frame::Connect { transaction_id, .. } => {
                info!("received a connect request, responding");
                let mut buf = BytesMut::new();
                // set a new connection id, should be saved
                let connection_id: u64 = random();

                // write to buffer
                buf.put_u32(0);
                buf.put_u32(transaction_id);
                buf.put_u64(connection_id);
                buf
            }
            Frame::Announce {
                transaction_id,
                info_hash,
                ..
            } => {
                // obtain a lock to the db
                let mut db = self.db.lock().unwrap();
                // get all available peers
                let peers = match db.get(&info_hash) {
                    Some(values) => values,
                    None => {
                        // we didn't find the info hash so we add this peer into the hash
                        let this_peer = Peer::from(self.from_addr);
                        db.insert(info_hash.clone(), vec![this_peer]);

                        db.get(&info_hash).unwrap()
                    }
                };

                // generate the response
                let mut buf = BytesMut::new();
                buf.put_u32(1);
                buf.put_u32(transaction_id);
                buf.put_u32(1);
                buf.put_u32(peers.len() as u32); // leechers FIX
                buf.put_u32(peers.len() as u32); // seeders FIX
                peers.iter().for_each(|peer| {
                    match peer.addr.ip() {
                        IpAddr::V4(ip) => buf.put_u32(u32::from(ip)),
                        IpAddr::V6(ip) => buf.put_u32(0),
                    }
                    buf.put_u16(peer.addr.port());
                });
                buf
            }
            Frame::Scrape { .. } => BytesMut::new(),
            Frame::Error => {
                info!("received a bad request");
                let mut buf = BytesMut::new();
                let transaction_id: u32 = random();

                buf.put_u32(3);
                buf.put_u32(transaction_id);
                buf.put(&b"invalid request"[..]);
                buf
            }
        };

        self.send(buf).await.unwrap();
        Ok(())
    }

    async fn send(&self, buf: BytesMut) -> crate::Result<()> {
        loop {
            self.connection.writable().await?;

            match self.connection.try_send_to(&buf, self.from_addr) {
                Ok(sent) => {
                    println!("sent {} bytes", sent);
                    break;
                }
                Err(ref e) if e.kind() == io::ErrorKind::WouldBlock => {
                    // Writable false positive.
                    continue;
                }
                Err(e) => panic!("problem"),
            }
        }
        Ok(())
    }
}

#[derive(Debug)]
pub struct Peer {
    pub addr: SocketAddr,
}

impl Peer {
    pub fn from(addr: SocketAddr) -> Self {
        Peer { addr }
    }
}
