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

impl InsertMessage {
    pub fn process(&self, session_prices: &mut SessionPrices) -> ProjectResult<()> {
        match session_prices.entry(self.timestamp) {
            std::collections::btree_map::Entry::Vacant(entry) => {
                entry.insert(self.price);
                Ok(())
            }
            std::collections::btree_map::Entry::Occupied(_) => Err("timestamp already exists"),
        }
    }
}

impl QueryMessage {
    pub fn process(&self, session_prices: &SessionPrices) -> ProjectResult<Price> {
        if self.min_time > self.max_time
            || session_prices.range(self.min_time..=self.max_time).count() == 0
        {
            return Ok(0);
        }

        let sum = session_prices
            .range(self.min_time..=self.max_time)
            .map(|(_ts, price)| *price as i64)
            .sum::<i64>();
        let length =
            match i64::try_from(session_prices.range(self.min_time..=self.max_time).count()) {
                Ok(v) => v,
                Err(_) => return Err("cannot compute average"),
            };

        Ok((sum / length) as i32)
    }
}

#[cfg(test)]
mod tests {
    use crate::{InsertMessage, QueryMessage, Request, Response, SessionPrices};

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

    #[test]
    fn process_a_session() {
        // Arrange
        let mut session_prices = SessionPrices::new();

        let i1 = InsertMessage {
            timestamp: 100,
            price: 100,
        };

        let i2 = InsertMessage {
            timestamp: 105,
            price: 102,
        };

        let i3 = InsertMessage {
            timestamp: 103,
            price: 102,
        };

        let q1 = QueryMessage {
            min_time: 100,
            max_time: 103,
        };

        // Act
        i1.process(&mut session_prices).unwrap();
        i2.process(&mut session_prices).unwrap();
        i3.process(&mut session_prices).unwrap();
        let mean = q1.process(&session_prices).unwrap();

        // Assert
        assert_eq!(mean, 101);
    }
}
