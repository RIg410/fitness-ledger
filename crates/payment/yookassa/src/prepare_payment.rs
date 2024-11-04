use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
pub(crate) struct Amount {
    pub value: String,
    pub currency: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub(crate) struct Confirmation {
    #[serde(rename = "type")]
    pub confirmation_type: String,
    pub return_url: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub(crate) struct PaymentRequest {
    pub amount: Amount,
    pub capture: bool,
    pub confirmation: Confirmation,
    pub description: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub(crate) struct PaymentResponse {
    pub id: String,
    pub status: Status,
    pub confirmation: Confirmation,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "snake_case")]
pub(crate) enum Status {
    Pending,
    WaitingForCapture,
    Succeeded,
    Canceled,
}
