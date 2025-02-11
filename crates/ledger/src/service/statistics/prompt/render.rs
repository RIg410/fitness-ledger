use chrono::NaiveDate;
use eyre::Result;
use model::statistics::{
    day::{TrainingStat, TrainingType},
    month::{MarketingStat, MonthStatistics, SubscriptionStat, TreasuryIO}, training::TrainingsStat,
};
use std::collections::{HashMap, HashSet};

pub fn render_statistic(state: &HashMap<NaiveDate, MonthStatistics>) -> Result<String> {
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
        trainigs.write(&month_stats.training)?;
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
        "training database:\n{}application database:\n{}subscription sales database:\n{}:cost and income base\n{}\nnow:{}",
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
    wtr: csv::Writer<Vec<u8>>,
}

impl TrainingWriter {
    pub fn new() -> Result<Self> {
        let mut wtr = csv::Writer::from_writer(vec![]);
        wtr.write_record(&[
            "training name",
            "start at",
            "clients count",
            "instructor name",
            "type of workout",
            "training room",
            "debited from clients' subscriptions",
            "instructor reward",
        ])?;
        /*
        
pub struct TrainingsStat {
    pub trainings: HashMap<String, TrainingStat>,
    pub instructors: HashMap<String, InstructorsStat>,
}

pub struct TrainingStat {
    pub trainings_count: u64,
    pub total_clients: u64,
    pub total_earned: i64,
}

pub struct InstructorsStat {
    pub total_trainings: u64,
    pub total_clients: u64,
    pub total_earned: i64,
}

         */
        Ok(Self { wtr })
    }

    pub fn write(&mut self, training: &TrainingsStat) -> Result<()> {
        // let tp = match training.tp {
        //     TrainingType::Group => "group",
        //     TrainingType::Personal => "personal",
        //     TrainingType::Rent => "sub rent",
        // };
        // let instructor = training
        //     .instructor
        //     .as_ref()
        //     .map(|instructor| instructor.clone())
        //     .unwrap_or_else(|| "-".to_string());
        // self.wtr.write_record(&[
        //     training.name.clone(),
        //     training.start_at.format("%Y-%m-%d %H:%M").to_string(),
        //     training.clients.to_string(),
        //     instructor,
        //     tp.to_string(),
        //     training.room.clone(),
        //     training.earned.to_string(),
        //     training.paid.to_string(),
        // ])?;

        Ok(())
    }

    pub fn finish(self) -> Result<String> {
        let buff = self.wtr.into_inner()?;
        Ok(String::from_utf8(buff)?)
    }
}
