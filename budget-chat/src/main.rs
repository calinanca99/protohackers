use std::{
    collections::HashMap,
    sync::{Arc, RwLock},
};

use anyhow::bail;
use tokio::{
    io::{AsyncBufReadExt, AsyncWriteExt, BufReader},
    net::{TcpListener, TcpStream},
    sync::Mutex,
};

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
struct Username(String);

impl Username {
    // The username must have at least one character and consist
    // entirely of alphanumeric characters.
    fn new(name: String) -> anyhow::Result<Self> {
        // Add validation
        Ok(Self(name))
    }
}

#[derive(Clone, Debug)]
struct Connection {
    stream: Arc<Mutex<TcpStream>>,
}

impl Connection {
    fn new(stream: Arc<Mutex<TcpStream>>) -> Self {
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

    fn get_users(&self) -> impl IntoIterator<Item = Username> {
        vec![]
    }

    fn add_user(&mut self, username: Username, connection: &Connection) -> anyhow::Result<()> {
        let mut state = self.active_users.write().unwrap();
        if state.insert(username, connection.clone()).is_some() {
            eprintln!("Username is taken");
            bail!("Username is taken");
        }

        Ok(())
    }

    fn remove_user(&mut self, username: Username) {
        todo!()
    }
}

async fn handle_connection(mut stream: TcpStream, mut db: Db) -> anyhow::Result<()> {
    stream
        .write_all("Welcome to budgetchat! What shall I call you?\n".as_bytes())
        .await?;
    stream.flush().await?;

    let buf_reader = BufReader::new(&mut stream);

    let username = match buf_reader.lines().next_line().await? {
        Some(username) => Username::new(username)?,
        None => {
            println!("Client disconnected");
            return Ok(());
        }
    };

    let stream = Arc::new(Mutex::new(stream));
    let connection = Connection::new(stream.clone());
    db.add_user(username, &connection)?;

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
                if let Err(e) = handle_connection(stream, active_users).await {
                    eprintln!("{e}")
                }
            }
            Err(e) => {
                eprintln!("{e}")
            }
        }
    }
}
