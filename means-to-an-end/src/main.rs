use std::{
    io::{Read, Write},
    net::{TcpListener, TcpStream},
    thread::{self, ThreadId},
};

use env_logger::Env;
use log::{debug, error, info};
use means_to_an_end::{Request, SessionPrices};
use utils::addr;

fn handle_connection(mut connection: TcpStream, tid: ThreadId) {
    info!(
        "{:?} - Established connection with: {:?}",
        tid,
        connection.peer_addr()
    );

    let mut session_prices = SessionPrices::new();
    loop {
        let mut buffer = [0; 9];
        if let Err(e) = connection.read_exact(&mut buffer) {
            debug!("{:?} - Buffer: {:?}", tid, buffer);
            error!(
                "{:?} - Cannot read from the socket. Dropping connection: {:?}",
                tid, e
            );
            return;
        };

        debug!("{:?} - Buffer: {:?}", tid, buffer);
        match Request::new(&buffer) {
            Ok(Request::Insert(insert_message)) => {
                match insert_message.process(&mut session_prices) {
                    Ok(_) => {
                        info!("{:?} - Processed insert message {:?}", tid, insert_message);
                        continue;
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
                        );
                        continue;
                    }
                    Err(e) => {
                        error!(
                            "{:?} - Cannot write to socket. Dropping connection: {:?}",
                            tid, e
                        );
                        return;
                    }
                },
                Err(e) => {
                    error!(
                        "{:?} - Cannot process query message {:?}. Dropping connection: {:?}",
                        tid, query_message, e
                    );
                    return;
                }
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
    info!("Started listening on: {:?}", addr());

    for stream in listener.incoming() {
        match stream {
            Ok(connection) => {
                thread::spawn(move || handle_connection(connection, thread::current().id()));
            }
            Err(e) => {
                error!("Could not establish connection: {:?}", e)
            }
        }
    }
}
