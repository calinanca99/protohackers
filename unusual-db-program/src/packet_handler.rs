use std::{net::SocketAddr, sync::Arc};

use tokio::net::UdpSocket;

use crate::{Db, InsertMessage, Message, RetrieveMessage, VERSION_SPECIAL_KEY};

pub struct PacketHandler {
    socket: Arc<UdpSocket>,
    peer: SocketAddr,
    buf: Vec<u8>,
    db: Db,
}

impl PacketHandler {
    pub fn new(socket: Arc<UdpSocket>, peer: SocketAddr, buf: &[u8], db: Db) -> Self {
        Self {
            socket,
            peer,
            buf: buf.to_vec(),
            db,
        }
    }

    pub async fn process(&self) -> anyhow::Result<()> {
        let s = String::from_utf8(self.buf.to_vec())?;
        let message = Message::new(s)?;

        match message {
            Message::Insert(InsertMessage { key, value }) => {
                if key.as_str() == VERSION_SPECIAL_KEY {
                    return Ok(());
                }
                self.db.set_value(key, value).await;
            }
            Message::Retrieve(RetrieveMessage { key }) => {
                let value = self.db.get_value(&key).await.unwrap_or("".to_string());
                self.socket
                    .send_to(format!("{}={}", key, value).as_bytes(), self.peer)
                    .await?;
            }
            Message::Version => {
                self.socket
                    .send_to("version=1.0".as_bytes(), self.peer)
                    .await?;
            }
        };

        Ok(())
    }
}
