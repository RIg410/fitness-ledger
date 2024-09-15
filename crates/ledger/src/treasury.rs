use chrono::{Local, Utc};
use eyre::Error;
use model::{
    decimal::Decimal,
    session::Session,
    treasury::{
        income::Income, outcome::Outcome, subs::SellSubscription, Event, Sell, TreasuryEvent,
        UserInfo,
    },
    user::User,
};
use mongodb::bson::oid::ObjectId;
use storage::treasury::TreasuryStore;
use tx_macro::tx;

use crate::logs::Logs;

#[derive(Clone)]
pub struct Treasury {
    store: TreasuryStore,
    logs: Logs,
}

impl Treasury {
    pub fn new(store: TreasuryStore, logs: Logs) -> Self {
        Treasury { store, logs }
    }

    pub(crate) async fn sell(
        &self,
        session: &mut Session,
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

    pub(crate) async fn presell(
        &self,
        session: &mut Session,
        seller: User,
        phone: String,
        sell: Sell,
    ) -> Result<(), Error> {
        let debit = sell.debit();

        let sub = SellSubscription {
            buyer: UserInfo::from_phone(phone),
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

    #[tx]
    pub async fn payment(
        &self,
        session: &mut Session,
        user: User,
        amount: Decimal,
        description: String,
        date_time: &chrono::DateTime<Local>,
    ) -> Result<(), Error> {
        self.logs
            .payment(session, user.id, amount, description.clone(), date_time)
            .await;
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

    #[tx]
    pub async fn deposit(
        &self,
        session: &mut Session,
        user: User,
        amount: Decimal,
        description: String,
        date_time: &chrono::DateTime<Local>,
    ) -> Result<(), Error> {
        self.logs
            .deposit(session, user.id, amount, description.clone(), date_time)
            .await;
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
