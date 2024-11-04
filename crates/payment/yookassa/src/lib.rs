pub mod intent;
pub mod prepare_payment;

use env::Env;
use eyre::Error;
use intent::Intent;
use model::decimal::Decimal;
use prepare_payment::{Amount, Confirmation, PaymentRequest, PaymentResponse};
use uuid::Uuid;

const BASE_URL: &str = "https://api.yookassa.ru/v3/payments";

pub struct Yookassa {
    api_key: String,
    shop_id: String,
    bot_url: String,
}

impl Yookassa {
    pub fn new(env: &Env) -> Self {
        Self {
            api_key: env.yookassa_token().to_owned(),
            shop_id: env.yookassa_shop_id().to_owned(),
            bot_url: env.bot_url().to_owned(),
        }
    }

    pub async fn prepare_payment(
        &self,
        price: Decimal,
        description: &str,
    ) -> Result<Intent, Error> {
        let amount = Amount {
            value: price.to_string(),
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
            description: description.to_owned(),
        };
        let id: Uuid = Uuid::new_v4();
        let id_key = id.to_string();

        let request = serde_json::to_value(&payment)?;
        let response = reqwest::Client::new()
            .post(BASE_URL)
            .header(self.shop_id.clone(), self.api_key.clone())
            .header("Idempotence-Key", id_key.clone())
            .json(&payment)
            .send()
            .await?;
        let response = response.json::<serde_json::Value>().await?;
        let payment_resp = serde_json::from_value::<PaymentResponse>(response.clone())?;

        Ok(Intent {
            ident: id_key,
            request,
            response,
            redirect_url: payment_resp.confirmation.return_url,
            price,
            description: description.to_owned(),
            payment_id: payment_resp.id,
        })
    }
}
