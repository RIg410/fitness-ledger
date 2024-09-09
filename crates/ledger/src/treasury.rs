use chrono::{Local, Utc};
use eyre::Error;
use model::{
    decimal::Decimal,
    subscription::Subscription,
    treasury::{
        income::Income,
        outcome::Outcome,
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
            buyer: buyer.into(),
            info: sell.into(),
        };

        let event = TreasuryEvent {
            id: ObjectId::new(),
            date_time: Utc::now(),
            event: Event::SellSubscription(sub),
            debit,
            credit: Decimal::zero(),
            user: seller.into(),
        };

        self.store.insert(session, event).await?;
        Ok(())
    }

    pub async fn payment(
        &self,
        session: &mut ClientSession,
        user: User,
        amount: Decimal,
        description: String,
        date_time: &chrono::DateTime<Local>,
    ) -> Result<(), Error> {
        let event = TreasuryEvent {
            id: ObjectId::new(),
            date_time: date_time.with_timezone(&Utc),
            event: Event::Outcome(Outcome { description }),
            debit: amount,
            credit: Decimal::zero(),
            user: user.into(),
        };

        self.store.insert(session, event).await?;
        Ok(())
    }

    pub async fn deposit(
        &self,
        session: &mut ClientSession,
        user: User,
        amount: Decimal,
        description: String,
        date_time: &chrono::DateTime<Local>,
    ) -> Result<(), Error> {
        let event = TreasuryEvent {
            id: ObjectId::new(),
            date_time: date_time.with_timezone(&Utc),
            event: Event::Income(Income { description }),
            debit: amount,
            credit: Decimal::zero(),
            user: user.into(),
        };

        self.store.insert(session, event).await?;
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
