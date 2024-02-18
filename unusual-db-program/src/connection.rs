use std::{net::SocketAddr, sync::Arc};

use tokio::net::UdpSocket;

use crate::{Db, InsertMessage, Message, RetrieveMessage, VERSION_SPECIAL_KEY};

pub struct Connection {
    socket: Arc<UdpSocket>,
    peer: Option<SocketAddr>,
    db: Db,
}

impl Connection {
    pub fn new(socket: Arc<UdpSocket>, db: Db) -> Self {
        Self {
            socket,
            peer: None,
            db,
        }
    }

    pub async fn process(&mut self) -> anyhow::Result<()> {
        let mut buf = [0; 1000];
        let (bytes, peer) = self.socket.recv_from(&mut buf).await?;
        self.peer = Some(peer);

        let s = String::from_utf8(buf[..bytes].to_vec())?;
        let message = Message::new(s)?;
        self.handle_message(message).await?;

        Ok(())
    }

    async fn handle_message(&self, message: Message) -> anyhow::Result<()> {
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
                    .send_to(format!("{}={}", key, value).as_bytes(), self.peer.unwrap())
                    .await?;
            }
            Message::Version => {
                self.socket
                    .send_to("version=1.0".as_bytes(), self.peer.unwrap())
                    .await?;
            }
        };

        Ok(())
    }
}
