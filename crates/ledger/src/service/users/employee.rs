use eyre::{bail, eyre, Result};
use model::{
    decimal::Decimal,
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
    ) -> Result<()> {
        let user = self
            .store
            .get(session, id)
            .await?
            .ok_or_else(|| eyre!("User not found"))?;
        let employee = user.employee.ok_or_else(|| eyre!("User is not a couch"))?;
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
    ) -> Result<()> {
        let user = self
            .store
            .get(session, id)
            .await?
            .ok_or_else(|| eyre!("User not found"))?;
        if user.employee.is_some() {
            bail!("Already instructor");
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
}
