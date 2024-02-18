use std::sync::Arc;

use tokio::net::UdpSocket;

use unusual_db_program::{Connection, Db};

async fn handle(socket: Arc<UdpSocket>, db: Db) -> anyhow::Result<()> {
    let mut connection = Connection::new(socket, db);
    connection.process().await?;

    Ok(())
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let addr = utils::addr();
    let socket = UdpSocket::bind(addr).await?;
    let socket = Arc::new(socket);

    let db = Db::new();

    loop {
        let socket = socket.clone();
        let db = db.clone();

        tokio::task::spawn(handle(socket, db));
    }
}
