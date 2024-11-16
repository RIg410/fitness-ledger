use std::{env::var, sync::Arc};

use dotenv::dotenv;
use eyre::{Context, Error};
use rand::prelude::Distribution as _;

#[derive(Clone)]
pub struct Env(Arc<EnvInner>);

#[derive(Clone)]
pub struct EnvInner {
    tg_token: String,
    mongo_url: String,
    rust_log: String,
    app_url: String,
    yookassa_token: String,
    yookassa_shop_id: String,
    bot_url: String,
    jwt_secret: String,
}

impl Env {
    pub fn tg_token(&self) -> &str {
        &self.0.tg_token
    }

    pub fn mongo_url(&self) -> &str {
        &self.0.mongo_url
    }

    pub fn rust_log(&self) -> &str {
        &self.0.rust_log
    }

    pub fn app_url(&self) -> &str {
        &self.0.app_url
    }

    pub fn yookassa_token(&self) -> &str {
        &self.0.yookassa_token
    }

    pub fn yookassa_shop_id(&self) -> &str {
        &self.0.yookassa_shop_id
    }

    pub fn bot_url(&self) -> &str {
        &self.0.bot_url
    }

    pub fn jwt_secret(&self) -> &str {
        &self.0.jwt_secret
    }

    pub fn load() -> Result<Env, Error> {
        if dotenv().ok().is_none() {
            log::info!("dotenv not found");
        }

        Ok(Env(Arc::new(EnvInner {
            tg_token: var("TG_TOKEN").context("TG_TOKEN is not set")?,
            mongo_url: var("MONGO_URL").context("MONGO_URL is not set")?,
            rust_log: var("RUST_LOG").context("RUST_LOG is not set")?,
            app_url: var("APP_URL").context("APP_URL is not set")?,
            yookassa_token: var("YOOKASSA_TOKEN").context("YOOKASSA_TOKEN is not set")?,
            yookassa_shop_id: var("YOOKASSA_SHOP_ID").context("YOOKASSA_TOKEN is not set")?,
            bot_url: var("BOT_URL").context("BOT_URL is not set")?,
            jwt_secret: var("JWT_SECRET").unwrap_or_else(|_| {
                let mut rng = rand::thread_rng();
                rand::distributions::Alphanumeric
                    .sample_iter(&mut rng)
                    .take(64)
                    .map(char::from)
                    .collect()
            }),
        })))
    }
}
