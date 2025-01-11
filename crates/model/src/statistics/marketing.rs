use std::collections::HashMap;

use crate::user::User;
use bson::oid::ObjectId;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone, Copy, Hash, PartialEq, Eq)]
pub enum ComeFrom {
    Unknown {},
    Website {},
    Instagram {},
    VK {},
    YandexMap {},
    YandexDirect {},
    DirectAdds {},
    VkAdds {},
    DoubleGIS {},
    Avito {},
}

impl ComeFrom {
    pub fn iter() -> impl Iterator<Item = ComeFrom> {
        [
            ComeFrom::Unknown {},
            ComeFrom::Website {},
            ComeFrom::Instagram {},
            ComeFrom::VK {},
            ComeFrom::YandexMap {},
            ComeFrom::YandexDirect {},
            ComeFrom::DirectAdds {},
            ComeFrom::VkAdds {},
            ComeFrom::DoubleGIS {},
            ComeFrom::Avito {},
        ]
        .iter()
        .copied()
    }
}

impl Default for ComeFrom {
    fn default() -> Self {
        ComeFrom::Unknown {}
    }
}

#[derive(Default)]
pub struct UsersStat {
    pub come_from: HashMap<ComeFrom, Vec<ObjectId>>,
    pub users_count: u64,
    pub users_without_subscriptions: Vec<ObjectId>,
}

impl UsersStat {
    pub fn extend(&mut self, user: &User) {
        self.come_from
            .entry(user.come_from)
            .or_default()
            .push(user.id);
        self.users_count += 1;

        if user
            .payer()
            .ok()
            .map(|p| p.has_subscription())
            .unwrap_or_default()
        {
            self.users_without_subscriptions.push(user.id);
        }
    }
}
