use std::{
    io::{BufReader, Read, Write},
    net::{TcpListener, TcpStream},
};

use env_logger::Env;
use log::{error, info};
use means_to_an_end::{Request, SessionPrices};
use utils::addr;

type Tid = Option<usize>;

fn handle_connection(mut connection: TcpStream, tid: Tid) {
    info!("Established connection with: {:?}", connection.peer_addr());

    let mut session_prices = SessionPrices::new();
    let mut reader = BufReader::new(connection.try_clone().unwrap());
    loop {
        let mut buffer = [0; 9];
        if let Err(e) = reader.read_exact(&mut buffer) {
            error!(
                "{:?} - Cannot read from the socket. Dropping connection: {:?}",
                tid, e
            );
            return;
        };

        match Request::new(&buffer) {
            Ok(Request::Insert(insert_message)) => {
                match insert_message.process(&mut session_prices) {
                    Ok(_) => {
                        info!("{:?} - Processed insert message {:?}", tid, insert_message);
                    }
                    Err(e) => {
                        error!(
                            "{:?} - Cannot process insert message {:?}. Dropping connection: {:?}",
                            tid, insert_message, e
                        );
                        return;
                    }
                }
            }
            Ok(Request::Query(query_message)) => match query_message.process(&session_prices) {
                Ok(mean) => match connection.write_all(mean.to_be_bytes().as_slice()) {
                    Ok(_) => {
                        info!(
                            "{:?} - Sent mean {:?} to {:?}",
                            tid,
                            mean,
                            connection.peer_addr()
                        )
                    }
                    Err(e) => {
                        error!(
                            "{:?} - Cannot write to socket. Dropping connection: {:?}",
                            tid, e
                        );
                    }
                },
                Err(_) => todo!(),
            },
            Err(e) => {
                error!(
                    "{:?} - Cannot parse request. Dropping connection: {:?}",
                    tid, e
                );
                return;
            }
        }
    }
}

fn main() {
    let env = Env::new().filter_or("LOG_LEVEL", "debug");
    env_logger::init_from_env(env);

    let listener = TcpListener::bind(addr()).expect("Cannot bind to address");
    info!("Started listening on: {:?}", listener);

    for stream in listener.incoming() {
        match stream {
            Ok(connection) => {
                rayon::spawn(move || handle_connection(connection, rayon::current_thread_index()))
            }
            Err(e) => {
                error!("Could not establish connection: {:?}", e)
            }
        }
    }
}
