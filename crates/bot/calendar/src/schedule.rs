use async_trait::async_trait;
use bot_core::{
    callback_data::Calldata as _,
    calldata,
    context::Context,
    widget::{Jmp, View},
};
use bot_trainigs::{
    program::list::ProgramList,
    schedule::{group::ScheduleTrainingPreset, personal::PersonalTrainingPreset},
};
use bot_viewer::day::fmt_date;
use chrono::{DateTime, Local};
use eyre::Error;
use model::rights::Rule;
use serde::{Deserialize, Serialize};
use teloxide::types::InlineKeyboardMarkup;

pub struct ScheduleView {
    date_time: DateTime<Local>,
}

impl ScheduleView {
    pub fn new(date_time: DateTime<Local>) -> Self {
        Self { date_time }
    }
}

#[async_trait]
impl View for ScheduleView {
    fn name(&self) -> &'static str {
        "ScheduleView"
    }

    async fn show(&mut self, ctx: &mut Context) -> Result<(), Error> {
        let txt = format!("Запланировать на *{}*", fmt_date(&self.date_time));
        let mut keymap = InlineKeyboardMarkup::default();

        if ctx.has_right(Rule::ScheduleGroupTraining) {
            keymap = keymap.append_row(Callback::Group.btn_row("Групповое занятие 🧘"));
        }
        if ctx.has_right(Rule::SchedulePersonalTraining) {
            keymap = keymap.append_row(Callback::Personal.btn_row("Персональное занятие 🏋️"));
        }
        if ctx.has_right(Rule::ScheduleSubRent) {
            keymap = keymap.append_row(Callback::SubRent.btn_row("Субаренда 💰"));
        }
        ctx.edit_origin(&txt, keymap).await?;
        Ok(())
    }
    async fn handle_callback(&mut self, ctx: &mut Context, data: &str) -> Result<Jmp, eyre::Error> {
        match calldata!(data) {
            Callback::Group => {
                ctx.ensure(Rule::ScheduleGroupTraining)?;
                let preset = ScheduleTrainingPreset::with_day(self.date_time);
                Ok(ProgramList::new(preset).into())
            }
            Callback::Personal => {
                ctx.ensure(Rule::SchedulePersonalTraining)?;
                let preset = if ctx.is_couch() && !ctx.has_right(Rule::SelectPersonalInstructor) {
                    PersonalTrainingPreset::with_day_and_instructor(self.date_time, ctx.me.id)
                } else {
                    PersonalTrainingPreset::with_day(self.date_time)
                };

                Ok(Jmp::Next(preset.into_next_view()))
            }
            Callback::SubRent => {
                ctx.ensure(Rule::ScheduleSubRent)?;
                // ctx.ensure(Rule::EditSchedule)?;
                // Ok(Jmp::to("SubRentView"))
                todo!()
            }
        }
    }
}

#[derive(Serialize, Deserialize)]
enum Callback {
    Group,
    Personal,
    SubRent,
}
