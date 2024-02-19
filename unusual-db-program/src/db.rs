use std::{collections::HashMap, sync::Arc};

use tokio::sync::RwLock;

#[derive(Clone)]
pub struct Db {
    data: Arc<RwLock<HashMap<String, String>>>,
}

impl Db {
    pub fn new() -> Self {
        Self {
            data: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    pub async fn get_value(&self, k: &str) -> Option<String> {
        let state = self.data.read().await;
        state.get(k).cloned()
    }

    pub async fn set_value(&self, k: String, v: String) {
        let mut state = self.data.write().await;
        state.insert(k, v);
    }
}

impl Default for Db {
    fn default() -> Self {
        Db::new()
    }
}
