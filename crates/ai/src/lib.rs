use eyre::{bail, Error};
use log::info;
use model::{Context, Model, RequestPayload, Response, ResponsePayload};
use reqwest::Client;

mod model;
pub use model::Context as AiContext;
pub use model::Model as AiModel;

#[derive(Clone)]
pub struct Ai {
    pub base_url: String,
    pub api_key: String,
}

impl Ai {
    pub fn new(base_url: String, api_key: String) -> Self {
        Self { base_url, api_key }
    }
}

impl Ai {
    pub async fn ask(
        &self,
        model: Model,
        message: String,
        context: &mut Context,
    ) -> Result<Response, Error> {
        let client = Client::new();

        let history = std::mem::take(&mut context.history);
        let payload = RequestPayload {
            message,
            api_key: self.api_key.clone(),
            history: if history.is_empty() {
                None
            } else {
                Some(history)
            },
        };

        info!("Sending request to AI: {:?}", payload);
        let response = client
            .post(format!("{}/{}", self.base_url, model.name()))
            .json(&payload)
            .send()
            .await?;
        info!("Received response from AI: {:?}", response);
        context.history = payload.history.unwrap_or_default();
        context.add_user_message(payload.message);

        if response.status().is_success() {
            let resp_json: ResponsePayload = response.json().await?;
            if resp_json.is_success {
                Ok(Response {
                    response: resp_json.response.unwrap_or_default(),
                    used_words_count: resp_json.used_words_count.unwrap_or_default(),
                })
            } else {
                bail!(resp_json
                    .error_message
                    .unwrap_or_else(|| "Unknown error".to_string()))
            }
        } else {
            bail!("HTTP error: {}", response.status())
        }
    }
}
