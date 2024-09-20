pub mod client_training;
pub mod create_training;
pub mod edit;
pub mod find_training;
pub mod schedule_process;
pub mod schedule_training;
pub mod view_training_proto;

// #[derive(Default)]
// pub struct TrainingMainView;

// #[async_trait]
// impl View for TrainingMainView {
//     async fn show(&mut self, ctx: &mut Context) -> Result<()> {
//         let (msg, keyboard) = render();
//         ctx.edit_origin(&msg, keyboard).await?;
//         Ok(())
//     }

//     async fn handle_message(
//         &mut self,
//         ctx: &mut Context,
//         message: &Message,
//     ) -> Result<Option<Widget>> {
//         ctx.delete_msg(message.id).await?;
//         Ok(None)
//     }

//     async fn handle_callback(&mut self, ctx: &mut Context, data: &str) -> Result<Option<Widget>> {
//         let cb = if let Some(cb) = Callback::from_data(data) {
//             cb
//         } else {
//             return Ok(None);
//         };
//         match cb {
//             Callback::MyTrainings => {
//                 return Ok(Some(Box::new(ClientTrainings::new(ctx.me.id, None))))
//             }
//             Callback::Schedule => {
//                 let widget = Box::new(CalendarView::new(
//                     WeekId::default(),
//                     Some(Box::new(TrainingMainView)),
//                     None,
//                     None,
//                 ));
//                 return Ok(Some(widget));
//             }
//             Callback::FindTraining => {
//                 let widget = Box::new(FindTraining::default());
//                 return Ok(Some(widget));
//             }
//         }
//     }
//     fn take(&mut self) -> Widget {
//         TrainingMainView.boxed()
//     }
// }

// pub fn render() -> (String, InlineKeyboardMarkup) {
//     let msg = "🤸🏻‍♂️  Подберем тренировку для вас:".to_owned();
//     let mut keymap = InlineKeyboardMarkup::default();
//     keymap = keymap.append_row(vec![InlineKeyboardButton::callback(
//         "🫶🏻 Мои тренировки",
//         Callback::MyTrainings.to_data(),
//     )]);
//     keymap = keymap.append_row(vec![InlineKeyboardButton::callback(
//         "📅  Календарь",
//         Callback::Schedule.to_data(),
//     )]);
//     keymap = keymap.append_row(vec![InlineKeyboardButton::callback(
//         "🔍 Найти тренировку",
//         Callback::FindTraining.to_data(),
//     )]);

//     (msg, keymap)
// }

// #[derive(Clone, Debug, Serialize, Deserialize)]
// pub enum Callback {
//     MyTrainings,
//     Schedule,
//     FindTraining,
// }
