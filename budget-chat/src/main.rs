use std::{
    collections::{hash_map::Entry, HashMap},
    fmt::Display,
    sync::Arc,
};

use anyhow::bail;
use tokio::{
    io::{AsyncBufReadExt, AsyncWriteExt, BufReader},
    net::{tcp::OwnedWriteHalf, TcpListener, TcpStream},
    sync::{Mutex, RwLock},
};

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
struct Username(String);

impl Username {
    // The username must have at least one character and consist
    // entirely of alphanumeric characters.
    fn new(name: String) -> anyhow::Result<Self> {
        // TODO: Add validation
        Ok(Self(name))
    }
}

impl Display for Username {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

type WriteStream = OwnedWriteHalf;

#[derive(Clone, Debug)]
struct Connection {
    stream: Arc<Mutex<WriteStream>>,
}

impl Connection {
    fn new(stream: Arc<Mutex<WriteStream>>) -> Self {
        Self { stream }
    }
}

type Users = HashMap<Username, Connection>;

#[derive(Clone, Debug)]
struct Db {
    active_users: Arc<RwLock<Users>>,
}

impl Db {
    fn new() -> Self {
        Self {
            active_users: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    async fn get_users(&self) -> Users {
        self.active_users.read().await.clone()
    }

    async fn add_user(
        &mut self,
        username: &Username,
        connection: &Connection,
    ) -> anyhow::Result<()> {
        let mut state = self.active_users.write().await;
        match state.entry(username.clone()) {
            Entry::Occupied(_) => {
                eprintln!("Username is taken");
                bail!("Username is taken");
            }
            Entry::Vacant(e) => e.insert(connection.clone()),
        };

        Ok(())
    }

    async fn remove_user(&mut self, username: &Username) {
        let mut state = self.active_users.write().await;
        state.remove(username);
    }
}

async fn handle_connection(stream: TcpStream, mut db: Db) -> anyhow::Result<()> {
    let (rs, mut ws) = stream.into_split();

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
    let active_users = db.get_users().await;
    for connection in active_users.values() {
        let mut stream = connection.stream.lock().await;
        stream
            .write_all(format!("* {} has entered the room\n", username).as_bytes())
            .await?;
    }

    // Present to current user who's in the room (if any)
    let active_users_names = active_users
        .keys()
        .map(|k| format!("{k}"))
        .collect::<Vec<String>>();

    let room_has_members = !active_users_names.is_empty();
    if room_has_members {
        let active_users_list = active_users_names.join(", ");
        ws.write_all(format!("* The room contains: {}\n", active_users_list).as_bytes())
            .await?;
    }

    let write_stream = Arc::new(Mutex::new(ws));
    let connection = Connection::new(write_stream.clone());
    db.add_user(&username, &connection).await?;

    while let Some(line) = buf_lines.next_line().await? {
        let users = db.get_users().await;
        let other_users = users
            .iter()
            .filter(|(u, _)| **u != username)
            .map(|(_, c)| c);
        for connection in other_users {
            let mut stream = connection.stream.lock().await;
            stream
                .write_all(format!("[{}] {}\n", username, line).as_bytes())
                .await?;
        }
    }

    db.remove_user(&username).await;
    let active_users = db.get_users().await;
    for connection in active_users.values() {
        let mut stream = connection.stream.lock().await;
        stream
            .write_all(format!("* {} has left the room\n", username).as_bytes())
            .await?;
    }

    Ok(())
}

#[tokio::main]
async fn main() {
    let addr = utils::addr();
    let listener = TcpListener::bind(addr)
        .await
        .expect("Cannot bind TCP listener");

    let active_users = Db::new();

    loop {
        match listener.accept().await {
            Ok((stream, _)) => {
                let active_users = active_users.clone();

                tokio::spawn(async move {
                    if let Err(e) = handle_connection(stream, active_users).await {
                        eprintln!("{e}");
                    }
                });
            }
            Err(e) => {
                eprintln!("{e}")
            }
        }
    }
}
