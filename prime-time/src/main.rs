use env_logger::Env;
use log::{debug, info};
use primes::is_prime;
use serde_json::{json, Value};
use std::{
    io::{Read, Write},
    net::{TcpListener, TcpStream},
    thread,
};
use utils::addr;

fn handle_malformed_request(stream: &mut TcpStream) {
    let malformed_response = json!({"result": "failure"}).to_string();
    debug!("Malformed response: {:?}", malformed_response);
    info!("Sending malformed response...");
    let _ = stream
        .write(malformed_response.as_bytes())
        .expect("Cannot write to TCP stream");
}

fn format_response(is_prime: bool) -> String {
    let json = json!({"method": "isPrime", "prime": is_prime});
    format!("{json}\n")
}

fn handle_connection(stream: &mut TcpStream) {
    info!("Established connection with: {:?}", stream.peer_addr());

    loop {
        let mut buffer = vec![0; 4096];
        // Opt: Read buffered here
        let bytes = stream.read(&mut buffer).expect("Cannot read TCP stream");

        let json_request = if let Some(b) = buffer.split(|c| *c == b'\n').nth(0) {
            b
        } else {
            handle_malformed_request(stream);
            return;
        };

        // Reading 0 bytes means the connection is closed
        if bytes == 0 {
            break;
        }

        let decoded_message = if let Ok(s) = String::from_utf8(json_request.to_vec()) {
            s
        } else {
            handle_malformed_request(stream);
            return;
        };

        debug!("Payload: {:?}", decoded_message);
        info!("Read {} bytes", bytes);

        let request: serde_json::Result<Value> = serde_json::from_str(&decoded_message);
        if request.is_err() {
            handle_malformed_request(stream);
            return;
        }

        let request = request.unwrap();

        if request.get("method").is_none() || request.get("number").is_none() {
            handle_malformed_request(stream);
            return;
        }

        let method = &request["method"];
        let number = &request["number"];

        if *method != json!("isPrime") || !number.is_number() {
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

        let response = format_response(is_prime);
        debug!("Response: {:?}", response);

        let _ = stream
            .write(response.as_bytes())
            .expect("Cannot write to TCP stream");
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
