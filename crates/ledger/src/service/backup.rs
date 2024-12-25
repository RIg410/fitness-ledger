use std::{
    io::{Cursor, Read, Write as _},
    sync::Arc,
};

use eyre::{Context, Error};
use log::info;
use model::session::Session;
use serde::de::DeserializeOwned;
use storage::{
    calendar::CalendarStore, history::HistoryStore, program::ProgramStore, requests::RequestStore,
    rewards::RewardsStore, subscription::SubscriptionsStore, treasury::TreasuryStore,
    user::UserStore, Storage,
};
use tx_macro::tx;
use zip::write::SimpleFileOptions;

const USERS: &str = "users.json";
const HISTORY: &str = "history.json";
const CALENDAR: &str = "calendar.json";


pub struct Backup {
    users: Arc<UserStore>,
    history: Arc<HistoryStore>,
    programs: Arc<ProgramStore>,
    calendar: Arc<CalendarStore>,
    rewards: Arc<RewardsStore>,
    subscriptions: Arc<SubscriptionsStore>,
    treasury: Arc<TreasuryStore>,
    requests: Arc<RequestStore>,
}

impl Backup {
    pub fn new(store: Storage) -> Backup {
        Backup {
            users: store.users,
            history: store.history,
            programs: store.programs,
            calendar: store.calendar,
            rewards: store.rewards,
            subscriptions: store.subscriptions,
            treasury: store.treasury,
            requests: store.requests,
        }
    }

    #[tx]
    pub async fn apply_backup(&self, session: &mut Session, dump: Vec<u8>) -> Result<(), Error> {
        log::info!("Applying backup");
        let mut zip = zip::ZipArchive::new(Cursor::new(dump))?;

        if zip.by_name(USERS).is_ok() {
            info!("Restoring users");
            self.users
                .restore_dump(session, self.read_file(&mut zip, USERS)?)
                .await
                .context("users")?;
            info!("Users restored");
        } else {
            log::warn!("No users in backup");
        }

        if zip.by_name(HISTORY).is_ok() {
            info!("Restoring history");
            self.history
                .restore_dump(session, self.read_file(&mut zip, HISTORY)?)
                .await
                .context("history")?;
            info!("History restored");
        } else {
            log::warn!("No history in backup");
        }

        if zip.by_name(CALENDAR).is_ok() {
            info!("Restoring calendar");
            self.calendar
                .restore_dump(session, self.read_file(&mut zip, CALENDAR)?)
                .await
                .context("calendar")?;
            info!("Calendar restored");
        } else {
            log::warn!("No calendar in backup");
        }

        log::info!("Backup applied");
        Ok(())
    }

    fn read_file<T>(
        &self,
        zip: &mut zip::ZipArchive<Cursor<Vec<u8>>>,
        name: &str,
    ) -> Result<T, Error>
    where
        T: DeserializeOwned,
    {
        let mut file = zip.by_name(name)?;
        let mut buf = Vec::new();
        file.read_to_end(&mut buf)?;
        let value = serde_json::from_slice(&buf).context(name.to_owned())?;
        Ok(value)
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

        zip.start_file(USERS, options)?;
        zip.write_all(&serde_json::to_vec_pretty(
            &self.users.dump(session).await.context("users")?,
        )?)?;

        zip.start_file(HISTORY, options)?;
        zip.write_all(&serde_json::to_vec_pretty(
            &self.history.dump(session).await.context("history")?,
        )?)?;

        zip.start_file(CALENDAR, options)?;
        zip.write_all(&serde_json::to_vec_pretty(
            &self.calendar.dump(session).await.context("calendar")?,
        )?)?;

        zip.start_file("programs.json", options)?;
        zip.write_all(&serde_json::to_vec_pretty(
            &self.programs.dump(session).await.context("programs")?,
        )?)?;

        zip.start_file("rewards.json", options)?;
        zip.write_all(&serde_json::to_vec_pretty(
            &self.rewards.dump(session).await.context("rewards")?,
        )?)?;

        zip.start_file("subscriptions.json", options)?;
        zip.write_all(&serde_json::to_vec_pretty(
            &self
                .subscriptions
                .dump(session)
                .await
                .context("subscriptions")?,
        )?)?;

        zip.start_file("treasury.json", options)?;
        zip.write_all(&serde_json::to_vec_pretty(
            &self.treasury.dump(session).await.context("treasury")?,
        )?)?;

        zip.start_file("request.json", options)?;
        zip.write_all(&serde_json::to_vec_pretty(
            &self.requests.dump(session).await.context("treasury")?,
        )?)?;

        let mut writer = zip.finish()?;
        writer.flush()?;
        log::info!("Backup done:{} kb", writer.get_ref().len() / 1024);
        Ok(writer.into_inner())
    }
}
