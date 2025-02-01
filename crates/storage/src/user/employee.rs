use super::UserStore;
use bson::oid::ObjectId;
use bson::{to_bson, to_document};
use eyre::{Error, Result};
use futures_util::TryStreamExt as _;
use log::info;
use model::session::Session;
use model::user::employee::Employee;
use model::user::rate::Rate;
use model::user::User;
use mongodb::bson::doc;

impl UserStore {
    pub async fn employees_with_ready_fix_reward(
        &self,
        session: &mut Session,
    ) -> Result<Vec<User>> {
        let filter = doc! {
            "employee.rates": { "$elemMatch": { "Fix.next_payment_date": { "$lte": chrono::Utc::now() } } }
        };
        let mut cursor = self.users.find(filter).session(&mut *session).await?;
        Ok(cursor.stream(&mut *session).try_collect().await?)
    }

    pub async fn update_employee_reward_and_rates(
        &self,
        session: &mut Session,
        id: ObjectId,
        reward: model::decimal::Decimal,
        update_rates: Option<Vec<Rate>>,
    ) -> std::result::Result<(), Error> {
        info!(
            "Updating couch reward: {:?} {:?} {:?}",
            id, reward, update_rates
        );

        let update = if let Some(rates) = update_rates {
            doc! {
                "$inc": { "version": 1 },
                "$set": { 
                    "employee.rates": to_bson(&rates)?,
                     "employee.reward":  reward.inner() 
                }
            }
        } else {
            doc! {
                "$set": { "employee.reward":  reward.inner() },
                "$inc": { "version": 1 },
            }
        };

        let result = self
            .users
            .update_one(doc! { "_id": id }, update)
            .session(&mut *session)
            .await?;
        if result.modified_count == 0 {
            return Err(Error::msg("employee not found"));
        }
        Ok(())
    }

    pub async fn set_employee(
        &self,
        session: &mut Session,
        id: ObjectId,
        couch: &Employee,
    ) -> Result<()> {
        info!("Setting employee for user {}: {:?}", id, couch);
        self.users
            .update_one(
                doc! { "_id": id },
                doc! { "$set": { "employee": to_document(couch)? }, "$inc": { "version": 1 } },
            )
            .session(&mut *session)
            .await?;
        Ok(())
    }

    pub async fn delete_employee(&self, session: &mut Session, id: ObjectId) -> Result<(), Error> {
        info!("Deleting employee: {:?}", id);
        let result = self
            .users
            .update_one(
                doc! { "_id": id },
                doc! { "$unset": { "employee": "" }, "$inc": { "version": 1 } },
            )
            .session(&mut *session)
            .await?;
        if result.modified_count == 0 {
            return Err(Error::msg("Couch not found"));
        }
        Ok(())
    }

    pub async fn employees(&self, session: &mut Session) -> Result<Vec<User>, Error> {
        let filter = doc! { "employee.role": { "$ne": null } };
        let mut cursor = self.users.find(filter).session(&mut *session).await?;
        Ok(cursor.stream(&mut *session).try_collect().await?)
    }
}
