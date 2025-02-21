use chrono::NaiveDate;
use eyre::Error;
use model::{
    session::Session,
    statistics::month::{EmployeeStat, MonthStatistics},
    treasury::Event,
    user::rate::EmployeeRole,
};

use crate::service::{treasury::Treasury, users::Users};

use super::aggregation::month_range;


pub async fn load_treasury(
    session: &mut Session,
    month_id: NaiveDate,
    treasury: &Treasury,
    users: &Users,
    stat: &mut MonthStatistics,
) -> Result<(), Error> {
    let (start, end) = month_range(&month_id);
    let mut rows = treasury.find_range(session, Some(start), Some(end)).await?;
    while let Some(row) = rows.next(session).await {
        let row = row?;
        let sum = row.sum().int_part().abs();
        match row.event {
            Event::SellSubscription(_) => {
                stat.treasury.sell_subscriptions += sum;
            }
            Event::Income(_) => {
                stat.treasury.income_other += sum;
            }
            Event::SubRent { .. } => {
                stat.treasury.rent += sum;
            }
            Event::Rent { .. } => {
                stat.treasury.rent += sum;
            }
            Event::Outcome(_) => {
                stat.treasury.other_expense += sum;
            }
            Event::Reward(user_id) => {
                if let Some(user) = users
                    .get(session, user_id.object_id().unwrap_or_default())
                    .await?
                {
                    if let Some(user_stat) =
                        stat.treasury.employees.iter_mut().find(|i| i.id == user.id)
                    {
                        user_stat.paid += sum;
                    } else {
                        stat.treasury.employees.push(EmployeeStat {
                            id: user.id,
                            name: user.name.first_name.clone(),
                            paid: sum,
                            role: user
                                .employee
                                .as_ref()
                                .map(|e| e.role.clone())
                                .unwrap_or_else(|| EmployeeRole::Couch),
                        });
                    }
                }
            }
            Event::Marketing(source) => {
                let source_sum = stat.treasury.marketing.entry(source).or_insert(0);
                *source_sum += sum;

                let stat = stat.marketing.source.entry(source).or_insert_with(|| {
                    model::statistics::month::SourceStat {
                        buy_test: 0,
                        buy_subscription: 0,
                        requests_count: 0,
                        earned: 0,
                        spent: 0,
                    }
                });
                stat.spent += sum;
            }
        }
    }

    Ok(())
}
