use std::{
    collections::{hash_map::Entry, HashMap},
    sync::Arc,
};

use anyhow::bail;
use tokio::sync::RwLock;

use crate::users::{UserStream, Username, Users};

#[derive(Clone, Debug)]
pub struct Db {
    active_users: Arc<RwLock<Users>>,
}

impl Db {
    pub fn new() -> Self {
        Self {
            active_users: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    pub async fn get_users(&self) -> Users {
        self.active_users.read().await.clone()
    }

    pub async fn add_user(
        &mut self,
        username: &Username,
        connection: &UserStream,
    ) -> anyhow::Result<()> {
        let mut state = self.active_users.write().await;
        match state.entry(username.clone()) {
            Entry::Occupied(_) => {
                eprintln!("Username is taken");
                bail!("Username is taken");
            }
            Entry::Vacant(e) => e.insert(connection.clone()),
        };

        Ok(())
    }

    pub async fn remove_user(&mut self, username: &Username) {
        let mut state = self.active_users.write().await;
        state.remove(username);
    }
}
