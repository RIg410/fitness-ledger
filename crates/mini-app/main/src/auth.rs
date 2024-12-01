use std::collections::BTreeMap;

use chrono::Utc;
use eyre::Error;
use hmac::{Hmac, Mac};
use log::info;
use sha2::Sha256;
type HmacSha256 = Hmac<Sha256>;

const TG_TTL: i64 = 60; // 60 seconds

#[derive(Clone)]
pub struct TgAuth {
    secret: [u8; 32],
}

impl TgAuth {
    pub fn new(tg_token: &str) -> Self {
        let mut sec_key = HmacSha256::new_from_slice("WebAppData".as_bytes()).unwrap();
        sec_key.update(tg_token.as_bytes());
        let mut key = [0u8; 32];
        key.copy_from_slice(&sec_key.finalize().into_bytes());
        TgAuth { secret: key }
    }

    pub fn validate(&self, mut query: BTreeMap<String, String>) -> Result<i64, Error> {
        info!("Validating telegram auth:{:?}", query);
        let original_hash = query.remove("hash").ok_or_else(|| eyre::eyre!("No hash"))?;
        let auth_date = query
            .get("auth_date")
            .ok_or_else(|| eyre::eyre!("No auth date"))?;
        let auth_date = auth_date.parse::<i64>()?;
        let user = query.get("user").ok_or_else(|| eyre::eyre!("No user"))?;
        let user: serde_json::Value = serde_json::from_str(&user)?;

        let tg_id = user
            .get("id")
            .ok_or_else(|| eyre::eyre!("No user id"))?
            .as_i64()
            .ok_or_else(|| eyre::eyre!("Invalid user id"))?;

        let now = Utc::now().timestamp();
        if auth_date > now || now - auth_date > TG_TTL {
            return Err(eyre::eyre!("Invalid auth date"));
        }

        let mut items = query
            .iter()
            .map(|(k, v)| format!("{k}={v}"))
            .collect::<Vec<_>>();
        items.sort();

        let data_check_string = items.join("\n");
        info!("data_check_string:{}", data_check_string);
        let mut secret = HmacSha256::new_from_slice(&self.secret)?;
        secret.update(data_check_string.as_bytes());
        let hash = hex::encode(secret.finalize().into_bytes());
        if original_hash != hash {
            return Err(eyre::eyre!("Invalid hash"));
        }

        Ok(tg_id)
    }
}
