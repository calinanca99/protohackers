use budget_chat::{Connection, Db};
use tokio::net::{TcpListener, TcpStream};

async fn handle_connection(stream: TcpStream, db: Db) -> anyhow::Result<()> {
    let connection = Connection::new(stream, db);
    connection.process().await?;

    Ok(())
}

#[tokio::main]
async fn main() {
    let addr = utils::addr();
    let listener = TcpListener::bind(addr)
        .await
        .expect("Cannot bind TCP listener");

    println!("Listening on: {addr}");

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
