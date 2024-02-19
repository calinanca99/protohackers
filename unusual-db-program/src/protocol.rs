pub const VERSION_SPECIAL_KEY: &str = "version";

#[derive(Debug, PartialEq)]
pub enum Message {
    Insert(InsertMessage),
    Retrieve(RetrieveMessage),
    Version,
}

impl Message {
    pub fn new(s: String) -> anyhow::Result<Self> {
        if s.contains('=') {
            let parts = s.splitn(2, '=').collect::<Vec<_>>();
            let key = parts[0].to_string();
            let value = parts[1].to_string();

            return Ok(Self::Insert(InsertMessage { key, value }));
        }

        match s.as_str().trim_start().trim_end() {
            VERSION_SPECIAL_KEY => Ok(Self::Version),
            _ => Ok(Self::Retrieve(RetrieveMessage { key: s })),
        }
    }
}

#[derive(Debug, PartialEq)]
pub struct InsertMessage {
    pub key: String,
    pub value: String,
}

#[derive(Debug, PartialEq)]
pub struct RetrieveMessage {
    pub key: String,
}

#[cfg(test)]
mod tests {
    use crate::protocol::*;

    #[test]
    fn parses_insert_with_one_equal() {
        let m = Message::new("foo=bar".to_string());
        assert_eq!(
            Message::Insert(InsertMessage {
                key: "foo".to_string(),
                value: "bar".to_string()
            }),
            m.unwrap()
        );
    }

    #[test]
    fn parses_insert_with_two_equals() {
        let m = Message::new("foo=bar=baz".to_string());
        assert_eq!(
            Message::Insert(InsertMessage {
                key: "foo".to_string(),
                value: "bar=baz".to_string()
            }),
            m.unwrap()
        );
    }

    #[test]
    fn parses_insert_with_empty_value() {
        let m = Message::new("foo=".to_string());
        assert_eq!(
            Message::Insert(InsertMessage {
                key: "foo".to_string(),
                value: "".to_string()
            }),
            m.unwrap()
        );
    }

    #[test]
    fn parses_insert_when_key_is_formed_of_equal_sings() {
        let m = Message::new("foo===".to_string());
        assert_eq!(
            Message::Insert(InsertMessage {
                key: "foo".to_string(),
                value: "==".to_string()
            }),
            m.unwrap()
        );
    }

    #[test]
    fn parses_insert_with_empty_key() {
        let m = Message::new("=foo".to_string());
        assert_eq!(
            Message::Insert(InsertMessage {
                key: "".to_string(),
                value: "foo".to_string()
            }),
            m.unwrap()
        );
    }

    #[test]
    fn parses_version() {
        let m = Message::new("version".to_string());
        assert_eq!(Message::Version, m.unwrap());
    }

    #[test]
    fn parses_retrieve() {
        let m = Message::new("foo".to_string());
        assert_eq!(
            Message::Retrieve(RetrieveMessage {
                key: "foo".to_string()
            }),
            m.unwrap()
        );
    }
}
