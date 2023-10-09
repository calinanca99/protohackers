use std::net::{TcpListener, TcpStream};

use log::{error, info};
use utils::addr;

fn handle_connection(connection: TcpStream) {
    info!("Established connection with: {:?}", connection.peer_addr());

    info!("Ending connection with: {:?}", connection.peer_addr());
}

fn main() {
    let listener = TcpListener::bind(addr()).expect("Cannot bind to address");

    for stream in listener.incoming() {
        match stream {
            Ok(connection) => rayon::spawn(move || handle_connection(connection)),
            Err(e) => {
                error!("Could not establish connection: {:?}", e)
            }
        }
    }
}
