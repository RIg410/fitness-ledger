use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
pub struct Amount {
    pub value: String,
    pub currency: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Confirmation {
    #[serde(rename = "type")]
    pub confirmation_type: String,
    pub return_url: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct PaymentRequest {
    pub amount: Amount,
    pub capture: bool,
    pub confirmation: Confirmation,
    pub description: String,
}
