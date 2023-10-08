use env_logger::Env;
use log::{debug, error, info};
use serde_json::{json, Value};
use std::{
    io::{Read, Write},
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
}

fn handle_connection(stream: &mut TcpStream, tid: ThreadId) {
    info!(
        "{:?} - Established connection with: {:?}",
        tid,
        stream.peer_addr()
    );

    loop {
        let mut buffer = vec![0; 4096];
        // Opt: Read buffered here
        let bytes = match stream.read(&mut buffer) {
            // Client is disconnected
            Ok(b) if b == 0 => break,
            Ok(b) => b,
            Err(e) => {
                error!("{:?} - Cannot read from socket: {:?}", tid, e);
                continue;
            }
        };

        let json_request = match buffer[..bytes].split(|c| *c == b'\n').nth(0) {
            Some(json) => json,
            None => {
                error!("{:?} - No JSON request in the payload", tid);
                handle_malformed_request(stream, tid);
                return;
            }
        };

        let decoded_message = match String::from_utf8(json_request.to_vec()) {
            Ok(message) => message,
            Err(e) => {
                error!("{:?} - Invalid UTF8: {:?}", tid, e);
                handle_malformed_request(stream, tid);
                return;
            }
        };

        debug!("{:?} - Payload: {:?}", tid, decoded_message);
        info!("{:?} - Read {} bytes", tid, bytes);

        match serde_json::from_str::<Value>(&decoded_message) {
            Ok(request) => {
                if request.get("method").is_none() || request.get("number").is_none() {
                    handle_malformed_request(stream, tid);
                    return;
                }

                let method = request["method"].clone();
                let number = request["number"].clone();

                if method != json!("isPrime") || !number.is_number() {
                    handle_malformed_request(stream, tid);
                    return;
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
                        continue;
                    }
                    Err(e) => {
                        error!("{:?} - Cannot write to socket: {:?}", tid, e);
                        continue;
                    }
                }
            }
            Err(e) => {
                error!("{:?} - Invalid JSON: {:?}", tid, e);
                handle_malformed_request(stream, tid);
                return;
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

    for mut stream in listener.incoming().flatten() {
        thread::spawn(move || handle_connection(&mut stream, thread::current().id()));
    }

    Ok(())
}
