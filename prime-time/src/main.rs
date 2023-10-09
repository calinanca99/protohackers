use env_logger::Env;
use log::{debug, error, info, warn};
use serde_json::{json, Value};
use std::{
    io::{BufRead, BufReader, Write},
    net::{TcpListener, TcpStream},
    thread::{self, ThreadId},
};
use utils::addr;

fn handle_malformed_request(stream: &mut TcpStream, tid: ThreadId) {
    let malformed_response = format!("{}\n", json!({"result": "failure"}));

    match stream.write_all(malformed_response.as_bytes()) {
        Ok(_) => {
            debug!("{:?} - Malformed response: {:?}", tid, malformed_response);
        }
        Err(e) => {
            error!("{:?} - Cannot write to socket: {:?}", tid, e);
        }
    }

    info!(
        "{:?} - Ending connection with: {:?}",
        tid,
        stream.peer_addr()
    );
}

fn handle_connection(mut stream: TcpStream, tid: ThreadId) {
    info!(
        "{:?} - Established connection with: {:?}",
        tid,
        stream.peer_addr()
    );

    let mut buffer = BufReader::new(stream.try_clone().unwrap());
    loop {
        let mut json_request = String::new();
        match buffer.read_line(&mut json_request) {
            Ok(b) if b == 0 => {
                warn!("{:?} - Client disconnected", tid);
                break;
            }
            Ok(b) => {
                info!("Read a JSON payload of size {} bytes", b)
            }
            Err(e) => {
                error!("{:?} - Cannot read from socket: {:?}", tid, e);
                continue;
            }
        }

        debug!("{:?} - Payload: {:?}", tid, json_request);

        match serde_json::from_str::<Value>(&json_request) {
            Ok(request) => {
                if request.get("method").is_none() || request.get("number").is_none() {
                    return handle_malformed_request(&mut stream, tid);
                }

                let method = request["method"].clone();
                let number = request["number"].clone();

                if method != json!("isPrime") || !number.is_number() {
                    return handle_malformed_request(&mut stream, tid);
                }

                debug!("{:?} - JSON number {:?}", tid, number);

                // At this point it's known that `number` is a valid JSON number
                debug!("{:?} - Checking if {:?} is prime", tid, number);
                let is_prime: bool = if number.is_f64() {
                    false
                } else if number.is_i64() {
                    let number = number.as_i64().unwrap();

                    if number < 0 {
                        false
                    } else {
                        // Any i64 larger than 0 fits in an u64
                        is_prime::is_prime((number as u64).to_string().as_ref())
                    }
                } else {
                    is_prime::is_prime(number.as_u64().unwrap().to_string().as_ref())
                };

                let response = format!("{}\n", json!({"method": "isPrime", "prime": is_prime}));

                match stream.write_all(response.as_bytes()) {
                    Ok(_) => {
                        debug!("{:?} - Response: {:?}", tid, response);
                    }
                    Err(e) => {
                        error!("{:?} - Cannot write to socket: {:?}", tid, e);
                    }
                }
            }
            Err(e) => {
                error!("{:?} - Invalid JSON: {:?}", tid, e);
                return handle_malformed_request(&mut stream, tid);
            }
        }
    }

    info!(
        "{:?} - Ending connection with: {:?}",
        tid,
        stream.peer_addr()
    );
}

fn main() -> std::io::Result<()> {
    let env = Env::new().filter_or("LOG_LEVEL", "debug");

    env_logger::init_from_env(env);

    let addr = addr();
    let listener = TcpListener::bind(addr)?;
    info!("Server listening on: {}", addr);

    for stream in listener.incoming().flatten() {
        thread::spawn(move || handle_connection(stream, thread::current().id()));
    }

    Ok(())
}
