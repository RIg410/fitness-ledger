use bson::oid::ObjectId;
use chrono::NaiveDate;
use eyre::Result;
use model::{
    rooms::Room,
    session::Session,
    statistics::{
        month::{MarketingStat, MonthStatistics, SubscriptionStat, TreasuryIO},
        training::TrainingsStat,
    },
};
use std::collections::{HashMap, HashSet};

use crate::service::users::Users;

pub async fn render_statistic(
    state: &HashMap<NaiveDate, MonthStatistics>,
    users: &Users,
    session: &mut Session,
) -> Result<String> {
    let mut trainigs = TrainingWriter::new()?;
    let mut marketing = MarketingWriter::new()?;
    let mut subscriptions = SubscriptionStatWriter::new()?;

    let employees = state
        .iter()
        .map(|(_, month)| &month.treasury.employees)
        .flat_map(|employees| employees.iter().map(|employee| employee.name.to_string()))
        .collect::<HashSet<_>>();
    let mut treasury = TreasuryIOWriter::new(employees)?;

    for (month, month_stats) in state.iter() {
        trainigs
            .write(month, &month_stats.training, users, session)
            .await?;
        marketing.write(month, &month_stats.marketing)?;
        subscriptions.write(month, &month_stats.subscriptions)?;
        treasury.write(month, &month_stats.treasury)?;
    }

    let trainings_db = trainigs.finish()?;
    let marketing_db = marketing.finish()?;
    let subscriptions_db = subscriptions.finish()?;
    let treasury_db = treasury.finish()?;

    let now = chrono::Local::now();
    Ok(format!(
        "training avigation:\n{}application avigation:\n{}subscription sales avigation:\n{}:cost and income base\n{}\nnow:{}",
        trainings_db, marketing_db, subscriptions_db, treasury_db, now.format("%Y-%m-%d %H:%M")
    ))
}

pub struct TreasuryIOWriter {
    wtr: csv::Writer<Vec<u8>>,
    employees: Vec<String>,
}

impl TreasuryIOWriter {
    pub fn new(employees: HashSet<String>) -> Result<Self> {
        let mut wtr = csv::Writer::from_writer(vec![]);

        let mut rows = vec![
            "month".to_string(),
            "rent payment".to_string(),
            "sublease payment received".to_string(),
            "other expenses".to_string(),
            "other income".to_string(),
            "received from sales of subscriptions".to_string(),
        ];
        let employees = employees.into_iter().collect::<Vec<_>>();
        for employee in &employees {
            rows.push(format!("{} salary", employee));
        }

        wtr.write_record(&rows)?;

        Ok(Self { wtr, employees })
    }

    pub fn write(&mut self, month: &NaiveDate, stat: &TreasuryIO) -> Result<()> {
        let mut row = vec![
            month.format("%Y-%m").to_string(),
            stat.rent.to_string(),
            stat.sub_rent.to_string(),
            stat.other_expense.to_string(),
            stat.income_other.to_string(),
            stat.sell_subscriptions.to_string(),
        ];

        for emp in self.employees.iter() {
            if let Some(employee) = stat.employees.iter().find(|i| i.name == *emp) {
                row.push(employee.paid.to_string());
            } else {
                row.push("0".to_string());
            }
        }

        self.wtr.write_record(&row)?;
        Ok(())
    }

    pub fn finish(self) -> Result<String> {
        let buff = self.wtr.into_inner()?;
        Ok(String::from_utf8(buff)?)
    }
}

pub struct SubscriptionStatWriter {
    wtr: csv::Writer<Vec<u8>>,
}

impl SubscriptionStatWriter {
    pub fn new() -> Result<Self> {
        let mut wtr = csv::Writer::from_writer(vec![]);
        wtr.write_record(&[
            "month",
            "subscription name",
            "number of subscription sold",
            "earned on sales",
            "user subscriptions burned out",
            "discounts were issued for the amount",
        ])?;
        Ok(Self { wtr })
    }

    pub fn write(&mut self, month: &NaiveDate, stat: &[SubscriptionStat]) -> Result<()> {
        for subscription in stat {
            self.wtr.write_record(&[
                month.format("%Y-%m").to_string(),
                subscription.name.clone(),
                subscription.count.to_string(),
                subscription.earned.to_string(),
                subscription.burned_training.to_string(),
                subscription.discount.to_string(),
            ])?;
        }

        Ok(())
    }

    pub fn finish(self) -> Result<String> {
        let buff = self.wtr.into_inner()?;
        Ok(String::from_utf8(buff)?)
    }
}

pub struct MarketingWriter {
    wtr: csv::Writer<Vec<u8>>,
}

impl MarketingWriter {
    pub fn new() -> Result<Self> {
        let mut wtr = csv::Writer::from_writer(vec![]);
        wtr.write_record(&[
            "month",
            "where did the client come from(direction)",
            "clients who purchased a trial lesson",
            "clients who purchased a subscription",
            "number of applications from this direction",
            "earned from this direction",
            "spent on advertising in this direction",
        ])?;
        Ok(Self { wtr })
    }

    pub fn write(&mut self, month: &NaiveDate, stat: &MarketingStat) -> Result<()> {
        for (source, source_stat) in stat.source.iter() {
            self.wtr.write_record(&[
                month.format("%Y-%m").to_string(),
                source.name().to_string(),
                source_stat.buy_test.to_string(),
                source_stat.buy_subscription.to_string(),
                source_stat.requests_count.to_string(),
                source_stat.earned.to_string(),
                source_stat.spent.to_string(),
            ])?;
        }

        Ok(())
    }

    pub fn finish(self) -> Result<String> {
        let buff = self.wtr.into_inner()?;
        Ok(String::from_utf8(buff)?)
    }
}

pub struct TrainingWriter {
    by_program: csv::Writer<Vec<u8>>,
    by_instructor: csv::Writer<Vec<u8>>,
    by_room: csv::Writer<Vec<u8>>,
    by_type: csv::Writer<Vec<u8>>,
    by_weekday: csv::Writer<Vec<u8>>,
    by_time: csv::Writer<Vec<u8>>,

    instructors: HashMap<ObjectId, String>,
}

impl TrainingWriter {
    pub fn new() -> Result<Self> {
        let mut by_program = csv::Writer::from_writer(vec![]);
        let mut by_instructor = csv::Writer::from_writer(vec![]);
        let mut by_room = csv::Writer::from_writer(vec![]);
        let mut by_type = csv::Writer::from_writer(vec![]);
        let mut by_weekday = csv::Writer::from_writer(vec![]);
        let mut by_time = csv::Writer::from_writer(vec![]);

        by_program.write_record(&[
            "training name",
            "month",
            "trainings count",
            "total clients",
            "total earned",
            "trainings with out clients",
            "canceled trainings",
        ])?;

        by_instructor.write_record(&[
            "instructor",
            "month",
            "trainings count",
            "total clients",
            "total earned",
            "trainings with out clients",
            "canceled trainings",
        ])?;

        by_room.write_record(&[
            "room",
            "month",
            "trainings count",
            "total clients",
            "total earned",
            "trainings with out clients",
            "canceled trainings",
        ])?;

        by_type.write_record(&[
            "training type",
            "month",
            "trainings count",
            "total clients",
            "total earned",
            "trainings with out clients",
            "canceled trainings",
        ])?;

        by_weekday.write_record(&[
            "weekday",
            "month",
            "trainings count",
            "total clients",
            "total earned",
            "trainings with out clients",
            "canceled trainings",
        ])?;

        by_time.write_record(&[
            "time",
            "month",
            "trainings count",
            "total clients",
            "total earned",
            "trainings with out clients",
            "canceled trainings",
        ])?;

        Ok(Self {
            by_program,
            by_instructor,
            by_room,
            by_type,
            by_weekday,
            by_time,
            instructors: HashMap::new(),
        })
    }

    pub async fn write(
        &mut self,
        month: &NaiveDate,
        training: &TrainingsStat,
        users: &Users,
        session: &mut Session,
    ) -> Result<()> {
        for (program, stat) in training.by_program.iter() {
            let program_name = training.programs.get(program).cloned().unwrap_or_default();
            self.by_program.write_record(&[
                program_name,
                month.format("%Y-%m").to_string(),
                stat.trainings_count.to_string(),
                stat.total_clients.to_string(),
                stat.total_earned.to_string(),
                stat.trainings_with_out_clients.to_string(),
                stat.canceled_trainings.to_string(),
            ])?;
        }

        for (instructor, stat) in training.by_instructor.iter() {
            let instructor = if let Some(inst) = self.instructors.get(instructor) {
                inst.to_string()
            } else {
                let user = users.get(session, *instructor).await?;
                if let Some(user) = user {
                    self.instructors
                        .insert(*instructor, user.name.first_name.to_string());
                    user.name.first_name.to_string()
                } else {
                    "unknown".to_string()
                }
            };
            self.by_instructor.write_record(&[
                instructor,
                month.format("%Y-%m").to_string(),
                stat.trainings_count.to_string(),
                stat.total_clients.to_string(),
                stat.total_earned.to_string(),
                stat.trainings_with_out_clients.to_string(),
                stat.canceled_trainings.to_string(),
            ])?;
        }

        for (room, stat) in training.by_room.iter() {
            self.by_room.write_record(&[
                Room::from(*room).to_string(),
                month.format("%Y-%m").to_string(),
                stat.trainings_count.to_string(),
                stat.total_clients.to_string(),
                stat.total_earned.to_string(),
                stat.trainings_with_out_clients.to_string(),
                stat.canceled_trainings.to_string(),
            ])?;
        }

        for (tp, stat) in training.by_type.iter() {
            self.by_type.write_record(&[
                tp.to_string(),
                month.format("%Y-%m").to_string(),
                stat.trainings_count.to_string(),
                stat.total_clients.to_string(),
                stat.total_earned.to_string(),
                stat.trainings_with_out_clients.to_string(),
                stat.canceled_trainings.to_string(),
            ])?;
        }

        for (weekday, stat) in training.by_weekday.iter() {
            self.by_weekday.write_record(&[
                weekday.to_string(),
                month.format("%Y-%m").to_string(),
                stat.trainings_count.to_string(),
                stat.total_clients.to_string(),
                stat.total_earned.to_string(),
                stat.trainings_with_out_clients.to_string(),
                stat.canceled_trainings.to_string(),
            ])?;
        }

        for (time, stat) in training.by_time.iter() {
            self.by_time.write_record(&[
                time.to_string(),
                month.format("%Y-%m").to_string(),
                stat.trainings_count.to_string(),
                stat.total_clients.to_string(),
                stat.total_earned.to_string(),
                stat.trainings_with_out_clients.to_string(),
                stat.canceled_trainings.to_string(),
            ])?;
        }

        Ok(())
    }

    pub fn finish(self) -> Result<String> {
        let by_program = String::from_utf8(self.by_program.into_inner()?)?;
        let by_instructor = String::from_utf8(self.by_instructor.into_inner()?)?;
        let by_room = String::from_utf8(self.by_room.into_inner()?)?;
        let by_type = String::from_utf8(self.by_type.into_inner()?)?;
        let by_weekday = String::from_utf8(self.by_weekday.into_inner()?)?;
        let by_time = String::from_utf8(self.by_time.into_inner()?)?;

        Ok(format!(
            "by program:\n{}\nby instructor:\n{}\nby room:\n{}\nby type:\n{}\nby weekday:\n{}\nby time:\n{}",
            by_program, by_instructor, by_room, by_type, by_weekday, by_time
        ))
    }
}
