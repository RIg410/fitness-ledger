use super::UserInfo;
use crate::subscription::Subscription;
use bson::oid::ObjectId;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct SubscriptionInfo {
    pub id: ObjectId,
    pub name: String,
    pub items: u32,
    pub price: u32,
    pub version: u32,
}

impl From<Subscription> for SubscriptionInfo {
    fn from(subscription: Subscription) -> Self {
        SubscriptionInfo {
            id: subscription.id,
            name: subscription.name,
            items: subscription.items,
            price: subscription.price,
            version: subscription.version,
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct SellSubscription {
    seller: UserInfo,
    buyer: UserInfo,
    info: SubscriptionInfo,
}
