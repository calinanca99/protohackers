use std::{
    io::{Read, Write},
    net::{TcpListener, TcpStream},
    thread::{self, ThreadId},
};

use env_logger::Env;
use log::{debug, error, info};
use means_to_an_end::{ProjectResult, Request, SessionPrices};
use utils::addr;

fn read_all<T: Read>(source: &mut T, buf: &mut [u8], size: usize) -> ProjectResult<()> {
    let mut read = 0;

    loop {
        match source.read(&mut buf[read..]) {
            Ok(b) if b == 0 => return Err("client closed connection"),
            Ok(b) => {
                read += b;
                if read > size {
                    break;
                }
            }
            Err(_) => return Err("cannot read socket data"),
        }
    }

    Ok(())
}

fn handle_connection(mut connection: TcpStream, tid: ThreadId) {
    info!(
        "{:?} - Established connection with: {:?}",
        tid,
        connection.peer_addr()
    );

    let mut session_prices = SessionPrices::new();
    loop {
        let mut buffer = [0; 512];
        if let Err(e) = read_all(&mut connection, &mut buffer, 9) {
            debug!("{:?} - Buffer: {:?}", tid, buffer);
            error!(
                "{:?} - Cannot read from the socket. Dropping connection: {:?}",
                tid, e
            );
            return;
        };

        debug!("{:?} - Buffer: {:?}", tid, &buffer[..9]);
        match Request::new(&buffer[..9]) {
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
