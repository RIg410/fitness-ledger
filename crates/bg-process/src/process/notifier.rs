use bot_main::BotApp;
use eyre::Error;
use ledger::Ledger;
use model::session::Session;

use super::training;

pub struct Notifier {
    pub ledger: Ledger,
    pub bot: BotApp,
}

impl Notifier {
    pub fn new(ledger: Ledger, bot: BotApp) -> Notifier {
        Notifier { ledger, bot }
    }

    pub async fn process(&self, session: &mut Session) -> Result<(), Error> {
        



        Ok(())
    }


    async fn notify_about_training(&self, session: &mut Session) -> Result<(), Error> {
        //let training_to_notify = self.ledger.calendar.get_training_to_notify(session.user_id).await?;
        Ok(())
    }
}

