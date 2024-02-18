use std::{collections::HashMap, fmt::Display, sync::Arc};

use anyhow::bail;
use tokio::{net::tcp::OwnedWriteHalf, sync::Mutex};

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct Username(String);

impl Username {
    pub fn new(name: String) -> anyhow::Result<Self> {
        let missing_name = name.is_empty(); /* This allows names with only whitespace */
        let not_alpha_numeric = name.chars().any(|c| !c.is_alphanumeric());

        if missing_name || not_alpha_numeric {
            bail!("Username is not valid")
        }

        Ok(Self(name))
    }
}

impl Display for Username {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

type WriteStream = Arc<Mutex<OwnedWriteHalf>>;

#[derive(Clone, Debug)]
pub struct UserStream {
    stream: WriteStream,
}

impl UserStream {
    pub fn new(stream: WriteStream) -> Self {
        Self { stream }
    }

    pub fn stream(&self) -> WriteStream {
        self.stream.clone()
    }
}

pub type Users = HashMap<Username, UserStream>;
