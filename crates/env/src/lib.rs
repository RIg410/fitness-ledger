use std::{env::var, sync::Arc};

use dotenv::dotenv;
use eyre::{Context, Error};

#[derive(Clone)]
pub struct Env(Arc<EnvInner>);

#[derive(Clone)]
pub struct EnvInner {
    tg_token: String,
    mongo_url: String,
    rust_log: String,
    mini_app_key: String,
    app_url: String,
    payment_provider_token: String,
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

    pub fn mini_app_key(&self) -> &str {
        &self.0.mini_app_key
    }

    pub fn app_url(&self) -> &str {
        &self.0.app_url
    }

    pub fn payment_provider_token(&self) -> &str {
        &self.0.payment_provider_token
    }

    pub fn load() -> Result<Env, Error> {
        if dotenv().ok().is_none() {
            log::info!("dotenv not found");
        }

        Ok(Env(Arc::new(EnvInner {
            tg_token: var("TG_TOKEN").context("TG_TOKEN is not set")?,
            mongo_url: var("MONGO_URL").context("MONGO_URL is not set")?,
            rust_log: var("RUST_LOG").context("RUST_LOG is not set")?,
            mini_app_key: var("MINI_APP_KEY").context("MINI_APP_KEY is not set")?,
            app_url: var("APP_URL").context("APP_URL is not set")?,
            payment_provider_token: var("PAYMENT_PROVIDER_TOKEN")
                .context("PAYMENT_PROVIDER_TOKEN is not set")?,
        })))
    }
}
