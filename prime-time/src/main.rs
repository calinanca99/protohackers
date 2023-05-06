#![allow(unused_imports)]
use std::{
    io::{Read, Write},
    net::{TcpListener, TcpStream},
    thread,
};

use utils::addr;

fn handle_connection(mut _stream: TcpStream) {
    todo!()
}

fn main() -> std::io::Result<()> {
    let addr = addr();
    let listener = TcpListener::bind(addr)?;
    println!("Server listening on: {}", addr);

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
