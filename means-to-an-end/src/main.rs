#![allow(dead_code)]

use std::{
    collections::BTreeMap,
    net::{TcpListener, TcpStream},
};

use env_logger::Env;
use log::{error, info};
use utils::addr;

struct InsertMessage {
    /// Number of seconds since the UNIX Epoch.
    timestamp: Timestamp,
    /// Price of the asset in pennies.
    price: Price,
}

struct QueryMessage {
    /// Earliest timestamp of the period.
    min_time: Timestamp,
    /// Latest timestamp of the period.
    max_time: Timestamp,
}

enum Request {
    Insert(InsertMessage),
    Query(QueryMessage),
}

struct Response {
    /// Represents the mean of the inserted prices with timestamps T, where
    /// `min_time` <= T <= `max_time`. If there are no samples, then the `mean`
    /// is 0.
    ///
    /// The `mean` is rounded down in case it's not an integer.
    mean: Price,
}

type Timestamp = i32;
type Price = i32;

/// Represents the prices associated with a session.
type SessionPrices = BTreeMap<Timestamp, Price>;

fn handle_connection(connection: TcpStream) {
    info!("Established connection with: {:?}", connection.peer_addr());

    info!("Ending connection with: {:?}", connection.peer_addr());
}

fn main() {
    let env = Env::new().filter_or("LOG_LEVEL", "debug");
    env_logger::init_from_env(env);

    let listener = TcpListener::bind(addr()).expect("Cannot bind to address");
    info!("Started listening on: {:?}", listener);

    for stream in listener.incoming() {
        match stream {
            Ok(connection) => rayon::spawn(move || handle_connection(connection)),
            Err(e) => {
                error!("Could not establish connection: {:?}", e)
            }
        }
    }
}
