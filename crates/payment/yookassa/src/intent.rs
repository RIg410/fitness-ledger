use model::decimal::Decimal;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
pub struct Intent {
    pub ident: String,
    pub request: serde_json::Value,
    pub response: serde_json::Value,
    pub redirect_url: String,
    pub price: Decimal,
    pub description: String,
    pub payment_id: String,
}