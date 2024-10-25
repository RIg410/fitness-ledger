use crate::decimal::Decimal;
use bson::oid::ObjectId;
use eyre::Error;
use serde::{Deserialize, Serialize};

// Without nds.
const VAT: u8 = 1;

#[derive(Serialize, Deserialize)]
pub struct Receipt {
    items: Vec<Item>,
}

#[derive(Serialize, Deserialize)]
pub struct Item {
    description: String,
    quantity: String,
    amount: Amount,
    vat_code: u8,
}

#[derive(Serialize, Deserialize)]
pub struct Amount {
    value: String,
    currency: String,
}

pub struct Invoice {
    pub title: String,
    price: Decimal,
    pub payload: String,
    pub description: String,
    pub currency: String,
}

impl Invoice {
    pub fn price(&self) -> u32 {
        self.price.inner() as u32
    }

    pub fn make_receipt(&self) -> Receipt {
        Receipt {
            items: vec![Item {
                description: self.title.clone(),
                quantity: "1".to_string(),
                amount: Amount {
                    value: self.price.inner().to_string(),
                    currency: self.currency.clone(),
                },
                vat_code: VAT,
            }],
        }
    }
}

impl<S: Sellable> From<(S, ObjectId)> for Invoice {
    fn from((sellable, user_id): (S, ObjectId)) -> Self {
        Invoice {
            title: sellable.title(),
            price: sellable.price(),
            payload: PaymentPayload::Subscription {
                user_id,
                subscription_id: sellable.item_id(),
            }
            .encode(),
            description: sellable.description(),
            currency: "RUB".to_string(),
        }
    }
}

pub trait Sellable {
    fn item_id(&self) -> ObjectId;
    fn title(&self) -> String;
    fn price(&self) -> Decimal;
    fn description(&self) -> String;
}

pub enum PaymentPayload {
    Subscription {
        user_id: ObjectId,
        subscription_id: ObjectId,
    },
}

impl PaymentPayload {
    pub fn id(self) -> u8 {
        match self {
            PaymentPayload::Subscription { .. } => 0,
        }
    }

    pub fn encode(&self) -> String {
        let mut buffer: Vec<u8> = Vec::new();
        match self {
            PaymentPayload::Subscription {
                user_id,
                subscription_id,
            } => {
                buffer.push(0);
                buffer.extend_from_slice(&user_id.bytes());
                buffer.extend_from_slice(&subscription_id.bytes());
            }
        }
        let checksum: u8 = buffer.iter().fold(0, |acc, &x| acc.wrapping_add(x));
        buffer.push(checksum);
        hex::encode(buffer)
    }

    pub fn decode(data: &str) -> Result<Self, Error> {
        let bytes = hex::decode(data)?;
        let bytes = bytes.as_slice();
        let id = bytes[0];
        let payload = match id {
            0 => {
                let mut buf = [0; 12];
                buf.copy_from_slice(&bytes[1..13]);
                let user_id = ObjectId::from_bytes(buf);

                buf.copy_from_slice(&bytes[13..25]);
                let subscription_id = ObjectId::from_bytes(buf);
                let checksum = bytes[25];

                let calculated_checksum: u8 =
                    bytes[..25].iter().fold(0, |acc, &x| acc.wrapping_add(x));
                if checksum != calculated_checksum {
                    return Err(Error::msg("Invalid checksum"));
                }
                PaymentPayload::Subscription {
                    user_id,
                    subscription_id,
                }
            }
            _ => return Err(Error::msg("Unknown payload type")),
        };
        Ok(payload)
    }
}
