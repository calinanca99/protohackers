use std::sync::Arc;

use tokio::{
    io::{AsyncBufReadExt, AsyncWriteExt, BufReader},
    net::TcpStream,
    sync::Mutex,
};

use crate::{
    db::Db,
    users::{UserStream, Username},
};

pub struct Connection {
    stream: TcpStream,
    db: Db,
}

impl Connection {
    pub fn new(stream: TcpStream, db: Db) -> Self {
        Self { stream, db }
    }

    pub async fn process(mut self) -> anyhow::Result<()> {
        let (rs, mut ws) = self.stream.into_split();

        ws.write_all("Welcome to budgetchat! What shall I call you?\n".as_bytes())
            .await?;

        let buf_reader = BufReader::new(rs);
        let mut buf_lines = buf_reader.lines();

        let username = match buf_lines.next_line().await? {
            Some(username) => Username::new(username)?,
            None => {
                println!("Client disconnected");
                return Ok(());
            }
        };

        // Announce chat that another user joined
        let active_users = self.db.get_users().await;
        for connection in active_users.values() {
            let stream = connection.stream();
            let mut stream = stream.lock().await;
            stream
                .write_all(format!("* {} has entered the room\n", username).as_bytes())
                .await?;
        }

        // Present to current user who's in the room (if any)
        let active_users_names = active_users
            .keys()
            .map(|k| format!("{k}"))
            .collect::<Vec<String>>();

        let active_users_list = active_users_names.join(", ");
        ws.write_all(format!("* The room contains: {}\n", active_users_list).as_bytes())
            .await?;

        let write_stream = Arc::new(Mutex::new(ws));
        let connection = UserStream::new(write_stream.clone());
        self.db.add_user(&username, &connection).await?;

        while let Some(line) = buf_lines.next_line().await? {
            let users = self.db.get_users().await;
            let other_users = users
                .iter()
                .filter(|(u, _)| **u != username)
                .map(|(_, c)| c);
            for connection in other_users {
                let stream = connection.stream();
                let mut stream = stream.lock().await;
                stream
                    .write_all(format!("[{}] {}\n", username, line).as_bytes())
                    .await?;
            }
        }

        self.db.remove_user(&username).await;
        let active_users = self.db.get_users().await;
        for connection in active_users.values() {
            let stream = connection.stream();
            let mut stream = stream.lock().await;
            stream
                .write_all(format!("* {} has left the room\n", username).as_bytes())
                .await?;
        }

        Ok(())
    }
}
