use chrono::{DateTime, Local, Utc};
use eyre::Error;
use model::{
    decimal::Decimal,
    session::Session,
    statistics::source::Source,
    subscription::Subscription,
    treasury::{
        aggregate::{AggIncome, AggOutcome, TreasuryAggregate},
        income::Income,
        outcome::Outcome,
        subs::{SellSubscription, UserId},
        Event, TreasuryEvent,
    },
};
use mongodb::bson::oid::ObjectId;
use storage::treasury::TreasuryStore;
use tx_macro::tx;

use std::{ops::Deref, sync::Arc};

use super::history::History;

#[derive(Clone)]
pub struct Treasury {
    store: Arc<TreasuryStore>,
    logs: History,
}

impl Treasury {
    pub fn new(store: Arc<TreasuryStore>, logs: History) -> Self {
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
        sub: Subscription,
        discount: Option<Decimal>,
    ) -> Result<(), Error> {
        let mut debit = sub.price;

        if let Some(discount) = discount {
            debit -= sub.price * discount;
        }

        let sub = SellSubscription {
            info: sub.into(),
            buyer_id: UserId::Id(buyer_id),
            discount,
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
    pub async fn payment_rent(&self, session: &mut Session, amount: Decimal) -> Result<(), Error> {
        let dt = Local::now();
        self.logs
            .payment(session, amount, "Аренда".to_string(), &dt)
            .await?;
        let event = TreasuryEvent {
            id: ObjectId::new(),
            date_time: dt.with_timezone(&Utc),
            event: Event::Rent {},
            debit: Decimal::zero(),
            credit: amount,
            actor: session.actor(),
        };

        self.store.insert(session, event).await?;
        Ok(())
    }

    #[tx]
    pub async fn pay_for_marketing(
        &self,
        session: &mut Session,
        amount: Decimal,
        come_from: Source,
    ) -> Result<(), Error> {
        let dt = Local::now();
        self.logs
            .payment(session, amount, "маркетинг".to_string(), &dt)
            .await?;
        let event = TreasuryEvent {
            id: ObjectId::new(),
            date_time: dt.with_timezone(&Utc),
            event: Event::Marketing(come_from),
            debit: Decimal::zero(),
            credit: amount,
            actor: session.actor(),
        };

        self.store.insert(session, event).await?;
        Ok(())
    }

    #[tx]
    pub async fn take_sub_rent(
        &self,
        session: &mut Session,
        amount: Decimal,
        description: String,
    ) -> Result<(), Error> {
        let dt = Local::now();
        self.logs
            .payment(session, amount, format!("Субаренда:{}", description), &dt)
            .await?;
        let event = TreasuryEvent {
            id: ObjectId::new(),
            date_time: dt.with_timezone(&Utc),
            event: Event::SubRent {
                description: description.clone(),
            },
            debit: amount,
            credit: Decimal::zero(),
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
        from: Option<DateTime<Local>>,
        to: Option<DateTime<Local>>,
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
                Event::SubRent { .. } => {
                    outcome.sub_rent.add(tx.debit);
                }
                Event::Rent { .. } => {
                    outcome.sub_rent.add(tx.credit);
                }
                Event::Marketing(come_from) => {
                    outcome
                        .marketing
                        .entry(come_from)
                        .or_default()
                        .add(tx.credit);
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
