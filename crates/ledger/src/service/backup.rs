use std::io::{Cursor, Write as _};

use eyre::Error;
use model::session::Session;
use storage::{
    calendar::CalendarStore, history::HistoryStore, pre_sell::PreSellStore, program::ProgramStore,
    rewards::RewardsStore, subscription::SubscriptionsStore, treasury::TreasuryStore,
    user::UserStore, Storage,
};
use tx_macro::tx;
use zip::write::SimpleFileOptions;

#[derive(Clone)]
pub struct Backup {
    users: UserStore,
    history: HistoryStore,
    programs: ProgramStore,
    calendar: CalendarStore,
    pre_sell: PreSellStore,
    rewards: RewardsStore,
    subscriptions: SubscriptionsStore,
    treasury: TreasuryStore,
}

impl Backup {
    pub fn new(store: Storage) -> Backup {
        Backup {
            users: store.users,
            history: store.history,
            programs: store.programs,
            calendar: store.calendar,
            pre_sell: store.presell,
            rewards: store.rewards,
            subscriptions: store.subscriptions,
            treasury: store.treasury,
        }
    }

    #[tx]
    pub async fn make_backup(&self, session: &mut Session) -> Result<Vec<u8>, Error> {
        log::info!("Making backup");
        let mut zip = zip::ZipWriter::new(Cursor::new(Vec::new()));

        let options = SimpleFileOptions::default()
            .compression_method(zip::CompressionMethod::Bzip2)
            .compression_level(Some(9))
            .large_file(true)
            .unix_permissions(0o755);

        zip.start_file("users.json", options)?;
        zip.write_all(&serde_json::to_vec_pretty(
            &self.users.dump(session).await?,
        )?)?;

        zip.start_file("history.json", options)?;
        zip.write_all(&serde_json::to_vec_pretty(
            &self.history.dump(session).await?,
        )?)?;

        zip.start_file("programs.json", options)?;
        zip.write_all(&serde_json::to_vec_pretty(
            &self.programs.dump(session).await?,
        )?)?;

        zip.start_file("calendar.json", options)?;
        zip.write_all(&serde_json::to_vec_pretty(
            &self.calendar.dump(session).await?,
        )?)?;

        zip.start_file("pre_sell.json", options)?;
        zip.write_all(&serde_json::to_vec_pretty(
            &self.pre_sell.dump(session).await?,
        )?)?;

        zip.start_file("rewards.json", options)?;
        zip.write_all(&serde_json::to_vec_pretty(
            &self.rewards.dump(session).await?,
        )?)?;

        zip.start_file("subscriptions.json", options)?;
        zip.write_all(&serde_json::to_vec_pretty(
            &self.subscriptions.dump(session).await?,
        )?)?;

        zip.start_file("treasury.json", options)?;
        zip.write_all(&serde_json::to_vec_pretty(
            &self.treasury.dump(session).await?,
        )?)?;

        let mut writer = zip.finish()?;
        writer.flush()?;
        log::info!("Backup done:{} kb", writer.get_ref().len() / 1024);
        Ok(writer.into_inner())
    }
}
