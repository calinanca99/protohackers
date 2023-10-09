use std::net::{TcpListener, TcpStream};

use env_logger::Env;
use log::{error, info};
use utils::addr;

fn handle_connection(connection: TcpStream) {
    info!("Established connection with: {:?}", connection.peer_addr());

    info!("Ending connection with: {:?}", connection.peer_addr());
}

fn main() {
    let env = Env::new().filter_or("LOG_LEVEL", "debug");
    env_logger::init_from_env(env);

    let listener = TcpListener::bind(addr()).expect("Cannot bind to address");
    info!("Started listening on: {:?}", listener);

    for stream in listener.incoming() {
        match stream {
            Ok(connection) => rayon::spawn(move || handle_connection(connection)),
            Err(e) => {
                error!("Could not establish connection: {:?}", e)
            }
        }
    }
}
