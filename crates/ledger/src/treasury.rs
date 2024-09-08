use chrono::Utc;
use eyre::Error;
use model::{
    decimal::Decimal,
    subscription::Subscription,
    treasury::{
        subs::{SellSubscription, SubscriptionInfo},
        Event, TreasuryEvent,
    },
    user::User,
};
use mongodb::{bson::oid::ObjectId, ClientSession};
use storage::treasury::TreasuryStore;

#[derive(Clone)]
pub struct Treasury {
    store: TreasuryStore,
}

impl Treasury {
    pub fn new(store: TreasuryStore) -> Self {
        Treasury { store }
    }

    pub async fn sell(
        &self,
        session: &mut ClientSession,
        seller: User,
        buyer: User,
        sell: Sell,
    ) -> Result<(), Error> {
        let debit = sell.debit();

        let sub = SellSubscription {
            seller: seller.into(),
            buyer: buyer.into(),
            info: sell.into(),
        };

        let event = TreasuryEvent {
            id: ObjectId::new(),
            date_time: Utc::now(),
            event: Event::SellSubscription(sub),
            debit,
            credit: Decimal::zero(),
        };

        self.store.insert(event).await?;
        Ok(())
    }
}

pub enum Sell {
    Sub(Subscription),
    Free(u32, Decimal),
}
impl Sell {
    fn debit(&self) -> Decimal {
        match self {
            Sell::Sub(sub) => sub.price,
            Sell::Free(_, price) => *price,
        }
    }
}

impl From<Sell> for SubscriptionInfo {
    fn from(value: Sell) -> Self {
        match value {
            Sell::Sub(sub) => sub.into(),
            Sell::Free(items, price) => SubscriptionInfo {
                id: ObjectId::new(),
                name: "free".to_string(),
                items,
                price,
                version: 0,
                free: true,
            },
        }
    }
}
