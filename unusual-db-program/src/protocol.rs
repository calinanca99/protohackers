pub const VERSION_SPECIAL_KEY: &str = "version";

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

pub struct InsertMessage {
    pub key: String,
    pub value: String,
}

pub struct RetrieveMessage {
    pub key: String,
}
