use std::sync::Arc;

use tokio::net::UdpSocket;

use unusual_db_program::{Db, PacketHandler, MAX_MESSAGE_SIZE_BYTES};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let addr = utils::addr();
    let socket = UdpSocket::bind(addr).await?;
    println!("Bound Udp socket to: {addr}");
    let socket = Arc::new(socket);

    let db = Db::new();

    loop {
        let socket = socket.clone();
        let db = db.clone();

        let mut buf = [0; MAX_MESSAGE_SIZE_BYTES];
        let (bytes, peer) = socket.recv_from(&mut buf).await?;
        println!("Read {} bytes from {:?}", bytes, peer);

        tokio::task::spawn(async move {
            let ph = PacketHandler::new(socket, peer, &buf[..bytes], db);
            if let Err(e) = ph.process().await {
                eprintln!("{e}")
            }
        });
    }
}
