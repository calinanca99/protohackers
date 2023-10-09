#![allow(dead_code)]

use std::collections::BTreeMap;

#[derive(Debug, PartialEq)]
pub struct InsertMessage {
    /// Number of seconds since the UNIX Epoch.
    timestamp: Timestamp,
    /// Price of the asset in pennies.
    price: Price,
}

#[derive(Debug, PartialEq)]
pub struct QueryMessage {
    /// Earliest timestamp of the period.
    min_time: Timestamp,
    /// Latest timestamp of the period.
    max_time: Timestamp,
}

#[derive(Debug, PartialEq)]
pub enum Request {
    Insert(InsertMessage),
    Query(QueryMessage),
}

pub struct Response {
    /// Represents the mean of the inserted prices with timestamps T, where
    /// `min_time` <= T <= `max_time`. If there are no samples, then the `mean`
    /// is 0.
    ///
    /// The `mean` is rounded down in case it's not an integer.
    mean: Price,
}

pub type Timestamp = i32;
pub type Price = i32;

/// Represents the prices associated with a session.
pub type SessionPrices = BTreeMap<Timestamp, Price>;

type ProjectResult<T> = Result<T, &'static str>;

impl Request {
    pub fn new(bytes: &[u8]) -> Result<Self, &'static str> {
        match bytes.first() {
            Some(t) if *t == b'I' => {
                let timestamp = Self::to_i32(&bytes[1..5])?;
                let price = Self::to_i32(&bytes[5..])?;

                Ok(Self::Insert(InsertMessage { timestamp, price }))
            }
            Some(t) if *t == b'Q' => {
                let min_time = Self::to_i32(&bytes[1..5])?;
                let max_time = Self::to_i32(&bytes[5..])?;

                if min_time > max_time {
                    return Err("min_time is higher than max_time");
                }

                Ok(Self::Query(QueryMessage { min_time, max_time }))
            }
            _ => Err("Unknown message type"),
        }
    }

    fn to_i32(bytes: &[u8]) -> ProjectResult<Timestamp> {
        match <[u8; 4]>::try_from(bytes) {
            Ok(bs) => Ok(i32::from_be_bytes(bs)),
            Err(_) => Err("cannot convert to i32"),
        }
    }
}

impl Response {
    fn to_bytes(&self) -> [u8; 4] {
        self.mean.to_be_bytes()
    }
}

#[cfg(test)]
mod tests {
    use crate::{InsertMessage, QueryMessage, Request, Response};

    #[test]
    fn parses_an_insert_message_to_a_request() {
        // Arrange
        let mut network_data = vec![b'I'];
        let ts = 12345_i32.to_be_bytes().to_vec();
        let price = 101_i32.to_be_bytes().to_vec();

        network_data.extend(ts);
        network_data.extend(price);

        // Act
        let req = Request::new(&network_data).unwrap();

        // Assert
        assert_eq!(
            req,
            Request::Insert(InsertMessage {
                timestamp: 12345,
                price: 101
            })
        );
    }

    #[test]
    fn parses_a_query_message_to_a_request() {
        // Arrange
        let mut network_data = vec![b'Q'];
        let min_ts = 100_i32.to_be_bytes().to_vec();
        let max_ts = 10000_i32.to_be_bytes().to_vec();

        network_data.extend(min_ts);
        network_data.extend(max_ts);

        // Act
        let req = Request::new(&network_data).unwrap();

        // Assert
        assert_eq!(
            req,
            Request::Query(QueryMessage {
                min_time: 100,
                max_time: 10000
            })
        );
    }

    #[test]
    fn converts_a_response_to_bytes() {
        let res = Response { mean: 5107 };

        let bytes = res.to_bytes();

        // 19 * 256 + 243 == 5107
        assert_eq!([0, 0, 19, 243], bytes)
    }
}
