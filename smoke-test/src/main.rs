use std::{
    io::{Read, Write},
    net::{TcpListener, TcpStream},
    thread,
};

use utils::addr;

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
