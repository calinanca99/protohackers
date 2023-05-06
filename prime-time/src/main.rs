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
    let _ = stream
        .write(malformed_response.as_bytes())
        .expect("Cannot write to TCP stream");

    stream
        .shutdown(std::net::Shutdown::Both)
        .expect("Cannot shutdown Read side");
}

fn format_response(is_prime: bool) -> String {
    json!({"method": "isPrime", "prime": is_prime}).to_string()
}

fn handle_connection(stream: &mut TcpStream) {
    let mut buffer = String::new();
    let _ = stream
        .read_to_string(&mut buffer)
        .expect("Cannot read TCP stream");

    let request: serde_json::Result<Value> = serde_json::from_str(&buffer);
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

    let _ = stream
        .write(format_response(is_prime).as_bytes())
        .expect("Cannot write to TCP stream");
}

fn main() -> std::io::Result<()> {
    let addr = addr();
    let listener = TcpListener::bind(addr)?;
    println!("Server listening on: {}", addr);

    for mut stream in listener.incoming().flatten() {
        thread::spawn(move || handle_connection(&mut stream));
    }

    Ok(())
}
