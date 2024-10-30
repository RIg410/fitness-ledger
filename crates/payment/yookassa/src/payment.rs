use std::collections::HashMap;

use serde::{Deserialize, Serialize};

use crate::payment_request::Amount;

// {
//   "id": "22e12f66-000f-5000-8000-18db351245c7",
//   "status": "waiting_for_capture",
//   "paid": true,
//   "amount": {
//     "value": "2.00",
//     "currency": "RUB"
//   },
//   "authorization_details": {
//     "rrn": "10000000000",
//     "auth_code": "000000",
//     "three_d_secure": {
//       "applied": true
//     }
//   },
//   "created_at": "2018-07-18T10:51:18.139Z",
//   "description": "Заказ №72",
//   "expires_at": "2018-07-25T10:52:00.233Z",
//   "metadata": {},
//   "payment_method": {
//     "type": "bank_card",
//     "id": "22e12f66-000f-5000-8000-18db351245c7",
//     "saved": false,
//     "card": {
//       "first6": "555555",
//       "last4": "4444",
//       "expiry_month": "07",
//       "expiry_year": "2022",
//       "card_type": "Mir",
//       "card_product": {
//         "code": "MCP",
//         "name": "MIR Privilege"
//       },
//       "issuer_country": "RU",
//       "issuer_name": "Sberbank"
//     },
//     "title": "Bank card *4444"
//   },
//   "recipient": {
//     "account_id": "100500",
//     "gateway_id": "100700"
//   },
//   "refundable": false,
//   "test": false,
//   "income_amount": {
//     "value": "1.97",
//     "currency": "RUB"
//   }
// }
//https://yookassa.ru/developers/payment-acceptance/getting-started/quick-start
#[derive(Serialize, Deserialize, Debug)]
struct Payment {
    id: String,
    status: Status,
    paid: bool,
    amount: Amount,
    authorization_details: Option<AuthorizationDetails>,
    created_at: String,
    description: Option<String>,
    expires_at: Option<String>,
    metadata: Option<HashMap<String, String>>,

    payment_method: Option<PaymentMethod>,
    recipient: Recipient,
    refundable: bool,
    test: bool,
    income_amount: Option<Amount>,
}

#[derive(Serialize, Deserialize, Debug)]
struct AuthorizationDetails {
    rrn: Option<String>,
    auth_code: Option<String>,
    three_d_secure: ThreeDSecure,
}

#[derive(Serialize, Deserialize, Debug)]
struct ThreeDSecure {
    applied: bool,
}

#[derive(Serialize, Deserialize, Debug)]
struct PaymentMethod {
    #[serde(rename = "type")]
    payment_type: String,
    id: String,
    saved: bool,
    card: Card,
    title: Option<String>,
}

#[derive(Serialize, Deserialize, Debug)]
struct Card {
    first6: String,
    last4: String,
    expiry_month: String,
    expiry_year: String,
    card_type: String,
    card_product: CardProduct,
    issuer_country: String,
    issuer_name: String,
}

#[derive(Serialize, Deserialize, Debug)]
struct CardProduct {
    code: String,
    name: String,
}

#[derive(Serialize, Deserialize, Debug)]
struct Recipient {
    account_id: String,
    gateway_id: String,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "snake_case")]
pub enum Status {
    Pending,
    WaitingForCapture,
    Succeeded,
    Canceled,
}
