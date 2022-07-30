use crate::action::Action;
use byteorder::{ByteOrder, NetworkEndian};
use bytes::BytesMut;
use tracing::{debug, info};

pub struct Parser {
    /// The bytes of the datagram
    buf: BytesMut,
}

impl Parser {
    // create a new parser instance which takes a buffer
    pub fn new(buf: BytesMut) -> Self {
        debug!("parser: buf_len = {}", buf.len());
        Parser { buf }
    }

    // parse the buffer
    pub async fn parse(&mut self) -> crate::Result<Frame> {
        let action = Action::new(self.buf.to_vec()).await;
        match action {
            Action::Connect => {
                debug!("parsing a connection request");
                // build connect frame
                let connect_frame = Frame::Connect {
                    connection_id: NetworkEndian::read_u64(&self.buf[0..8]),
                    transaction_id: NetworkEndian::read_u32(&self.buf[12..16]),
                };
                Ok(connect_frame)
            }
            Action::Announce => {
                debug!("parsing a announce request");
                // build announce frame
                let announce_frame = Frame::Announce {
                    connection_id: NetworkEndian::read_u64(&self.buf[0..8]),
                    transaction_id: NetworkEndian::read_u32(&self.buf[12..16]),
                    info_hash: String::from_utf8_lossy(&self.buf[16..36].to_vec())
                        .parse()
                        .unwrap(),
                    peer_id: String::from_utf8_lossy(&self.buf[36..56].to_vec())
                        .parse()
                        .unwrap(),
                    downloaded: NetworkEndian::read_u64(&self.buf[56..64]),
                    left: NetworkEndian::read_u64(&self.buf[64..72]),
                    uploaded: NetworkEndian::read_u64(&self.buf[72..80]),
                    event: Event::from(NetworkEndian::read_u32(&self.buf[80..84])),
                    ip_address: NetworkEndian::read_u32(&self.buf[84..88]),
                    key: NetworkEndian::read_u32(&self.buf[88..92]),
                    num_want: NetworkEndian::read_i32(&self.buf[92..96]),
                    port: NetworkEndian::read_u16(&self.buf[96..98]),
                };

                Ok(announce_frame)
            }
            Action::Scrape => {
                let connection_id = NetworkEndian::read_u64(&self.buf[0..8]);
                let transaction_id = NetworkEndian::read_u32(&self.buf[12..16]);
                let info_hashes: Vec<String> = self.buf[16..]
                    .chunks_exact(8)
                    .map(|chunk| String::from_utf8_lossy(chunk).parse().unwrap())
                    .collect();

                Ok(Frame::Scrape {
                    connection_id,
                    transaction_id,
                    info_hashes,
                })
            }
            Action::Error => Ok(Frame::Error),
        }
    }
}

pub enum Frame {
    Connect {
        connection_id: u64,
        transaction_id: u32,
    },
    Announce {
        connection_id: u64,
        transaction_id: u32,
        info_hash: String,
        peer_id: String,
        downloaded: u64,
        left: u64,
        uploaded: u64,
        event: Event,
        ip_address: u32,
        key: u32,
        num_want: i32,
        port: u16,
    },
    Scrape {
        connection_id: u64,
        transaction_id: u32,
        info_hashes: Vec<String>,
    },
    Error,
}

pub enum Event {
    None,
    Completed,
    Started,
    Stopped,
}

impl Event {
    fn from(n: u32) -> Self {
        match n {
            1 => Event::Completed,
            2 => Event::Started,
            3 => Event::Stopped,
            _ => Event::None,
        }
    }
}
