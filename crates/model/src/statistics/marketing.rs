use std::collections::HashMap;

use bson::oid::ObjectId;
use serde::{Deserialize, Serialize};

use crate::user::User;

#[derive(Debug, Serialize, Deserialize, Clone, Copy, Hash, PartialEq, Eq)]
pub enum ComeFrom {
    Unknown {},
    Website {},
    Instagram {},
    VK {},
    YandexMap {},
    DirectAdds {},
    VkAdds {},
    DoubleGIS {},
}

impl Default for ComeFrom {
    fn default() -> Self {
        ComeFrom::Unknown {}
    }
}

pub struct UsersStat {
    pub come_from: HashMap<ComeFrom, Vec<ObjectId>>,
    pub users_count: u64,
    pub users_without_subscriptions: Vec<ObjectId>,
}

impl UsersStat {
    pub fn extend(&mut self, user: &User) {
        self.come_from
            .entry(user.come_from)
            .or_insert_with(Vec::new)
            .push(user.id.clone());
        self.users_count += 1;
        if !user.subscriptions.is_empty() {
            self.users_without_subscriptions.push(user.id.clone());
        }
    }
}

impl Default for UsersStat {
    fn default() -> Self {
        Self {
            come_from: Default::default(),
            users_count: 0,
            users_without_subscriptions: Default::default(),
        }
    }
}
