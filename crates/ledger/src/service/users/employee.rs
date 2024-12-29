use eyre::Result;
use model::{
    decimal::Decimal,
    errors::LedgerError,
    session::Session,
    user::{
        employee::Employee,
        rate::{EmployeeRole, Rate},
    },
};
use mongodb::bson::oid::ObjectId;
use tx_macro::tx;

use super::Users;

impl Users {
    #[tx]
    pub async fn update_employee_description(
        &self,
        session: &mut Session,
        id: ObjectId,
        description: String,
    ) -> Result<(), LedgerError> {
        let user = self
            .store
            .get(session, id)
            .await?
            .ok_or_else(|| LedgerError::UserNotFound(id))?;
        let employee = user
            .employee
            .ok_or_else(|| LedgerError::UserNotEmployee { user_id: id })?;
        let employee = Employee {
            description: description.clone(),
            reward: employee.reward,
            rates: employee.rates,
            role: employee.role,
        };

        self.store.set_employee(session, user.id, &employee).await?;
        Ok(())
    }

    #[tx]
    pub async fn make_user_employee(
        &self,
        session: &mut Session,
        id: ObjectId,
        description: String,
        rates: Vec<Rate>,
        role: EmployeeRole,
    ) -> Result<(), LedgerError> {
        let user = self
            .store
            .get(session, id)
            .await?
            .ok_or_else(|| LedgerError::UserNotFound(id))?;
        if user.employee.is_some() {
            return Err(LedgerError::UserAlreadyEmployee { user_id: id });
        }

        let employee = Employee {
            description,
            reward: Decimal::zero(),
            role,
            rates,
        };
        self.store.set_employee(session, id, &employee).await?;
        Ok(())
    }

    #[tx]
    pub async fn remove_rate(
        &self,
        session: &mut Session,
        id: ObjectId,
        rate: Rate,
    ) -> Result<(), LedgerError> {
        let user = self
            .store
            .get(session, id)
            .await?
            .ok_or_else(|| LedgerError::UserNotFound(id))?;
        let mut employee = user
            .employee
            .ok_or_else(|| LedgerError::UserNotEmployee { user_id: id })?;

        let rates = employee.rates.len();
        employee.rates.retain(|r| r != &rate);

        if rates == employee.rates.len() {
            return Err(LedgerError::RateNotFound { user_id: id, rate });
        }

        self.store.set_employee(session, user.id, &employee).await?;
        Ok(())
    }

    #[tx]
    pub fn update_rate(
        &self,
        session: &mut Session,
        id: ObjectId,
        old_date: Rate,
        new_rate: Rate,
    ) -> Result<(), LedgerError> {
        let user = self
            .store
            .get(session, id)
            .await?
            .ok_or_else(|| LedgerError::UserNotFound(id))?;
        let mut employee = user
            .employee
            .ok_or_else(|| LedgerError::UserNotEmployee { user_id: id })?;

        let rates = employee.rates.len();
        employee.rates.retain(|r| r != &old_date);

        if rates == employee.rates.len() {
            return Err(LedgerError::RateNotFound {
                user_id: id,
                rate: old_date,
            });
        }

        for rate in &mut employee.rates {
            if rate.as_u8() == old_date.as_u8() {
                return Err(LedgerError::RateTypeAlreadyExists {
                    user_id: id,
                    rate: new_rate,
                });
            }
        }

        employee.rates.push(new_rate);
        self.store.set_employee(session, user.id, &employee).await?;
        Ok(())
    }

    #[tx]
    pub async fn add_rate(
        &self,
        session: &mut Session,
        id: ObjectId,
        rate: Rate,
    ) -> Result<(), LedgerError> {
        let user = self
            .store
            .get(session, id)
            .await?
            .ok_or_else(|| LedgerError::UserNotFound(id))?;
        let mut employee = user
            .employee
            .ok_or_else(|| LedgerError::UserNotEmployee { user_id: id })?;

        if employee.rates.iter().any(|r| r.as_u8() == rate.as_u8()) {
            return Err(LedgerError::RateTypeAlreadyExists { user_id: id, rate });
        }

        employee.rates.push(rate);
        self.store.set_employee(session, user.id, &employee).await?;
        Ok(())
    }
}
