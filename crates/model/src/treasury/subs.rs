use super::UserInfo;
use crate::{decimal::Decimal, subscription::Subscription};
use bson::oid::ObjectId;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct SubscriptionInfo {
    pub id: ObjectId,
    pub name: String,
    pub items: u32,
    pub price: Decimal,
    pub version: u32,
    pub free: bool,
}

impl From<Subscription> for SubscriptionInfo {
    fn from(subscription: Subscription) -> Self {
        SubscriptionInfo {
            id: subscription.id,
            name: subscription.name,
            items: subscription.items,
            price: subscription.price,
            version: subscription.version,
            free: false,
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct SellSubscription {
    pub buyer: UserInfo,
    pub info: SubscriptionInfo,
}