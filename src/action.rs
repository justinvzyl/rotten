use byteorder::{ByteOrder, NetworkEndian};

#[derive(PartialEq, Debug)]
pub(crate) enum Action {
    Connect,
    Announce,
    Scrape,
    Error,
}

impl Action {
    pub async fn new(buf: Vec<u8>) -> Self {
        if buf.is_empty() || buf.len() < 16 {
            return Action::Error;
        }

        match NetworkEndian::read_u32(&buf[8..12]) {
            0 => Action::Connect,
            1 => Action::Announce,
            2 => Action::Scrape,
            _ => Action::Error,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use bytes::{BufMut, BytesMut};

    #[tokio::test]
    async fn test_action_connect() {
        let mut buf = BytesMut::new();
        buf.put_u64(0x41727101980);
        buf.put_u32(0);
        buf.put_u32(123);
        let action = Action::new(buf.to_vec()).await;
        assert_eq!(Action::Connect, action);
    }

    #[tokio::test]
    async fn test_action_announce() {
        let mut buf = BytesMut::new();
        buf.put_u64(0x41727101980);
        buf.put_u32(1);
        buf.put_u32(123);
        let action = Action::new(buf.to_vec()).await;
        assert_eq!(Action::Announce, action);
    }

    #[tokio::test]
    async fn test_action_scrape() {
        let mut buf = BytesMut::new();
        buf.put_u64(0x41727101980);
        buf.put_u32(2);
        buf.put_u32(123);
        let action = Action::new(buf.to_vec()).await;
        assert_eq!(Action::Scrape, action);
    }

    #[tokio::test]
    async fn test_action_error() {
        let mut buf = BytesMut::new();
        buf.put_u64(0x41727101980);
        buf.put_u32(3);
        buf.put_u32(123);
        let action = Action::new(buf.to_vec()).await;
        assert_eq!(Action::Error, action);
    }
}
