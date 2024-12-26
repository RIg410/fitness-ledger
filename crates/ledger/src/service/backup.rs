use std::{
    collections::HashMap,
    io::{Cursor, Read, Write as _},
};

use eyre::{Context, Error};
use model::session::Session;
use storage::{CollectionBackup, Storage};
use tx_macro::tx;
use zip::write::SimpleFileOptions;

pub struct Backup {
    store: Storage,
}

impl Backup {
    pub fn new(store: Storage) -> Backup {
        Backup { store }
    }

    #[tx]
    pub async fn apply_backup(&self, session: &mut Session, dump: Vec<u8>) -> Result<(), Error> {
        log::info!("Applying backup");
        let mut zip = zip::ZipArchive::new(Cursor::new(dump))?;

        let mut collections = HashMap::new();

        let names: Vec<_> = zip.file_names().map(|n| n.to_string()).collect();

        for name in names {
            if name.ends_with(".json") {
                let value = self.read_file(&mut zip, &name)?;
                collections.insert(name.trim_end_matches(".json").to_string(), value);
            }
        }

        self.store.restore(collections, session).await?;

        log::info!("Backup applied");
        Ok(())
    }

    fn read_file(
        &self,
        zip: &mut zip::ZipArchive<Cursor<Vec<u8>>>,
        name: &str,
    ) -> Result<CollectionBackup, Error> {
        let mut file = zip.by_name(name)?;
        let mut buf = Vec::new();
        file.read_to_end(&mut buf)?;
        Ok(bson::from_slice(&buf).context(name.to_owned())?)
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

        let backup = self.store.backup(session).await?;
        for (name, data) in backup {
            zip.start_file(format!("{}.json", name), options)?;
            zip.write_all(&bson::to_vec(&data)?)?;
        }

        let mut writer = zip.finish()?;
        writer.flush()?;
        log::info!("Backup done:{} kb", writer.get_ref().len() / 1024);
        Ok(writer.into_inner())
    }
}
