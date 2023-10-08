use env_logger::Env;
use log::{debug, error, info};
use primes::is_prime;
use serde_json::{json, Value};
use std::{
    io::{Read, Write},
    net::{TcpListener, TcpStream},
    thread,
};
use utils::addr;

fn handle_malformed_request(stream: &mut TcpStream) {
    let malformed_response = format!("{}\n", json!({"result": "failure"}));

    debug!("Malformed response: {:?}", malformed_response);

    let _ = stream
        .write(malformed_response.as_bytes())
        .expect("Cannot write to TCP stream");
}

fn handle_connection(stream: &mut TcpStream) {
    info!("Established connection with: {:?}", stream.peer_addr());

    loop {
        let mut buffer = vec![0; 4096];
        // Opt: Read buffered here
        let bytes = match stream.read(&mut buffer) {
            // Client is disconnected
            Ok(b) if b == 0 => break,
            Ok(b) => b,
            Err(e) => {
                error!("{:?}", e);
                continue;
            }
        };

        let json_request = match buffer[..bytes].split(|c| *c == b'\n').nth(0) {
            Some(json) => json,
            None => {
                error!("No JSON request in the payload");
                handle_malformed_request(stream);
                return;
            }
        };

        let decoded_message = match String::from_utf8(json_request.to_vec()) {
            Ok(message) => message,
            Err(e) => {
                error!("{:?}", e);
                handle_malformed_request(stream);
                return;
            }
        };

        debug!("Payload: {:?}", decoded_message);
        info!("Read {} bytes", bytes);

        match serde_json::from_str::<Value>(&decoded_message) {
            Ok(request) => {
                if request.get("method").is_none() || request.get("number").is_none() {
                    handle_malformed_request(stream);
                    return;
                }

                let method = request["method"].clone();
                let number = request["number"].clone();

                if method != json!("isPrime") || !number.is_number() {
                    handle_malformed_request(stream);
                    return;
                }

                // At this point it's known that `number` is a valid JSON number
                debug!("Checking if {:?} is prime", number);
                let is_prime: bool = if number.is_f64() {
                    false
                } else if number.is_i64() {
                    let number = number.as_i64().unwrap();

                    if number < 0 {
                        false
                    } else {
                        // Any i64 larger than 0 fits in an u64
                        is_prime(number as u64)
                    }
                } else {
                    is_prime(number.as_u64().unwrap())
                };

                let response = format!("{}\n", json!({"method": "isPrime", "prime": is_prime}));
                debug!("Response: {:?}", response);

                let _ = stream
                    .write(response.as_bytes())
                    .expect("Cannot write to TCP stream");
            }
            Err(e) => {
                error!("{:?}", e);
                handle_malformed_request(stream);
                return;
            }
        }
    }

    info!("Ending connection with: {:?}", stream.peer_addr());
}

fn main() -> std::io::Result<()> {
    let env = Env::new().filter_or("LOG_LEVEL", "debug");

    env_logger::init_from_env(env);

    let addr = addr();
    let listener = TcpListener::bind(addr)?;
    info!("Server listening on: {}", addr);

    for mut stream in listener.incoming().flatten() {
        thread::spawn(move || handle_connection(&mut stream));
    }

    Ok(())
}
