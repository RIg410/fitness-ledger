pub mod payment;
pub mod payment_request;

use ::model::invoice::Invoice;
use env::Env;
use eyre::Error;
use payment_request::{Amount, Confirmation, PaymentRequest};
use uuid::Uuid;

const BASE_URL: &str = "https://api.yookassa.ru/v3/payments";

pub struct YooKassa {
    check_store: (),
    api_key: String,
    shop_id: String,
    bot_url: String,
}

impl YooKassa {
    pub fn new(check_store: (), env: &Env) -> Self {
        Self {
            check_store,
            api_key: env.yookassa_token().to_owned(),
            shop_id: env.yookassa_shop_id().to_owned(),
            bot_url: env.bot_url().to_owned(),
        }
    }

    pub async fn prepare_payment(&self, invoice: &Invoice) -> Result<(), Error> {
        let amount = Amount {
            value: invoice.price.to_string(),
            currency: "RUB".to_owned(),
        };
        let confirmation = Confirmation {
            confirmation_type: "redirect".to_owned(),
            return_url: self.bot_url.to_owned(),
        };
        let payment = PaymentRequest {
            amount,
            capture: true,
            confirmation,
            description: invoice.description.to_owned(),
        };
        let id = Uuid::new_v4();
        let id_key = id.to_string();
        let response = reqwest::Client::new()
            .post(BASE_URL)
            .header(self.shop_id.clone(), self.api_key.clone())
            .header("Idempotence-Key", id_key.clone())
            .json(&payment)
            .send()
            .await?;
        let response = response.json::<serde_json::Value>().await?;
        todo!()
    }
}
