use chrono::{DateTime, Local, Utc};
use eyre::Error;
use model::{
    decimal::Decimal,
    session::Session,
    treasury::{
        aggregate::{AggIncome, AggOutcome, TreasuryAggregate},
        income::Income,
        outcome::Outcome,
        subs::{SellSubscription, UserId},
        Event, Sell, TreasuryEvent,
    },
};
use mongodb::bson::oid::ObjectId;
use storage::treasury::TreasuryStore;
use tx_macro::tx;

use std::ops::Deref;

use super::history::History;

#[derive(Clone)]
pub struct Treasury {
    store: TreasuryStore,
    logs: History,
}

impl Treasury {
    pub fn new(store: TreasuryStore, logs: History) -> Self {
        Treasury { store, logs }
    }

    pub async fn page(
        &self,
        session: &mut Session,
        limit: u64,
        offset: u64,
    ) -> Result<Vec<TreasuryEvent>, Error> {
        self.store.list(session, limit, offset).await
    }

    pub(crate) async fn sell(
        &self,
        session: &mut Session,
        buyer_id: ObjectId,
        sell: Sell,
    ) -> Result<(), Error> {
        let debit = sell.debit();
        let sub = SellSubscription {
            info: sell.into(),
            buyer_id: UserId::Id(buyer_id),
        };

        let event = TreasuryEvent {
            id: ObjectId::new(),
            date_time: Utc::now(),
            event: Event::SellSubscription(sub),
            debit,
            credit: Decimal::zero(),
            actor: session.actor(),
        };
        self.store.insert(session, event).await?;
        Ok(())
    }

    pub(crate) async fn presell(
        &self,
        session: &mut Session,
        phone: String,
        sell: Sell,
    ) -> Result<(), Error> {
        let debit = sell.debit();

        let sub = SellSubscription {
            buyer_id: UserId::Phone(phone),
            info: sell.into(),
        };

        let event = TreasuryEvent {
            id: ObjectId::new(),
            date_time: Utc::now(),
            event: Event::SellSubscription(sub),
            debit,
            credit: Decimal::zero(),
            actor: session.actor(),
        };
        self.store.insert(session, event).await?;
        Ok(())
    }

    #[tx]
    pub async fn payment(
        &self,
        session: &mut Session,
        amount: Decimal,
        description: String,
        date_time: &chrono::DateTime<Local>,
    ) -> Result<(), Error> {
        self.logs
            .payment(session, amount, description.clone(), date_time)
            .await?;
        let event = TreasuryEvent {
            id: ObjectId::new(),
            date_time: date_time.with_timezone(&Utc),
            event: Event::Outcome(Outcome { description }),
            debit: Decimal::zero(),
            credit: amount,
            actor: session.actor(),
        };

        self.store.insert(session, event).await?;
        Ok(())
    }

    #[tx]
    pub async fn deposit(
        &self,
        session: &mut Session,
        amount: Decimal,
        description: String,
        date_time: &chrono::DateTime<Local>,
    ) -> Result<(), Error> {
        self.logs
            .deposit(session, amount, description.clone(), date_time)
            .await?;
        let event = TreasuryEvent {
            id: ObjectId::new(),
            date_time: date_time.with_timezone(&Utc),
            event: Event::Income(Income { description }),
            debit: amount,
            credit: Decimal::zero(),
            actor: session.actor(),
        };

        self.store.insert(session, event).await?;
        Ok(())
    }

    pub(crate) async fn reward_employee(
        &self,
        session: &mut Session,
        to: UserId,
        amount: Decimal,
        date_time: &chrono::DateTime<Local>,
    ) -> Result<(), Error> {
        let event = TreasuryEvent {
            id: ObjectId::new(),
            date_time: date_time.with_timezone(&Utc),
            event: Event::Reward(to),
            debit: Decimal::zero(),
            credit: amount,
            actor: session.actor(),
        };

        self.store.insert(session, event).await?;
        Ok(())
    }

    pub async fn aggregate(
        &self,
        session: &mut Session,
        from: DateTime<Local>,
        to: DateTime<Local>,
    ) -> Result<TreasuryAggregate, Error> {
        let txs = self.store.range(session, from, to).await?;
        let mut debit = Decimal::zero();
        let mut credit = Decimal::zero();
        let mut income = AggIncome::default();
        let mut outcome = AggOutcome::default();

        let mut from = txs
            .first()
            .map(|tx| tx.date_time.with_timezone(&Local))
            .unwrap_or_else(Local::now);
        let mut to = from;

        for tx in txs {
            from = from.min(tx.date_time.with_timezone(&Local));
            to = to.max(tx.date_time.with_timezone(&Local));
            debit += tx.debit;
            credit += tx.credit;
            match tx.event {
                Event::SellSubscription(_) => {
                    income.subscriptions.add(tx.debit);
                }
                Event::Reward(_) => {
                    outcome.rewards.add(tx.credit);
                }
                Event::Outcome(_) => {
                    outcome.other.add(tx.credit);
                }
                Event::Income(_) => {
                    income.other.add(tx.debit);
                }
            }
        }

        Ok(TreasuryAggregate {
            from,
            to,
            debit,
            credit,
            income,
            outcome,
        })
    }
}

impl Deref for Treasury {
    type Target = TreasuryStore;

    fn deref(&self) -> &Self::Target {
        &self.store
    }
}
