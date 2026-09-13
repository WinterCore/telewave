use serde_json::Value;

use crate::{DB, tdtypes::Supergroup};

pub struct ChannelManager {
    db: DB,
}

impl ChannelManager {
    pub fn new() -> Self {
        Self {
            db: DB::new(),
        }
    }

    pub fn handle_update_supergroup(&self, supergroup: &Supergroup) {
        // supergroup.status
    }
}
