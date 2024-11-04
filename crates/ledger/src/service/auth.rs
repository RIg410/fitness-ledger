use std::sync::Arc;

use super::users::Users;
use eyre::eyre;
use eyre::Result;
use model::{auth::AuthKey, session::Session, user::User};
use mongodb::bson::oid::ObjectId;
use storage::auth_key::AuthKeys;
use tx_macro::tx;

pub struct AuthService {
    auth: Arc<AuthKeys>,
    users: Users,
}

impl AuthService {
    pub fn new(auth: Arc<AuthKeys>, users: Users) -> Self {
        Self { auth, users }
    }

    #[tx]
    pub async fn gen_key(&self, session: &mut Session, user_id: ObjectId) -> Result<AuthKey> {
        if let Some(auth_key) = self.auth.get(session, user_id).await? {
            let now = chrono::Utc::now();
            if now - auth_key.created_at > chrono::Duration::days(20) {
                let key = AuthKey::gen(user_id);
                self.auth.insert(session, &key).await?;
                Ok(key)
            } else {
                Ok(auth_key)
            }
        } else {
            let key = AuthKey::gen(user_id);
            self.auth.insert(session, &key).await?;
            Ok(key)
        }
    }

    pub async fn auth(&self, session: &mut Session, key: &str) -> Result<User> {
        let auth_key = self.auth.get_by_key(session, key).await?;
        let auth_key = auth_key.ok_or_else(|| eyre!("Auth key not found"))?;
        let user = self.users.get(session, auth_key.user_id).await?;
        user.ok_or_else(|| eyre!("User not found"))
    }
}
