use std::{
    env,
    io::{Read, Write},
    net::{TcpListener, TcpStream},
    thread,
};

fn handle_connection(mut stream: TcpStream) {
    // Read from stream
    let mut buffer = vec![];

    let bytes = stream
        .read_to_end(&mut buffer)
        .expect("Cannot read TCP stream");

    let addr = stream.local_addr();

    println!("Read: {} bytes from {:?}.", bytes, addr);

    // Write to stream
    let _ = stream.write(&buffer).expect("Cannot write to TCP stream");
}

fn addr() -> &'static str {
    match env::var("ENV") {
        Ok(var) if var == "PROTO" => "0.0.0.0:8000",
        Ok(_) | Err(_) => "127.0.0.1:8000",
    }
}

fn main() -> std::io::Result<()> {
    let addr = addr();
    let listener = TcpListener::bind(addr)?;
    println!("Echo-server listening on: {}", addr);

    for stream in listener.incoming() {
        match stream {
            Ok(stream) => {
                thread::spawn(|| handle_connection(stream));
            }
            Err(_) => { /* connection failed */ }
        }
    }
    Ok(())
}
