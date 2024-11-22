use axum::{
    extract::{Query, Request, State},
    http::StatusCode,
    middleware::Next,
    response::{IntoResponse as _, Response},
};

use bot_core::{
    bot::{Origin, TgBot, ValidToken},
    context::Context,
};
use bot_main::BotApp;
use chrono::{Duration, Utc};
use eyre::Error;
use ledger::Ledger;
use log::warn;
use serde::{Deserialize, Serialize};
use std::{collections::BTreeMap, sync::Arc, u64};
use teloxide::types::{ChatId, MessageId};
use tokio::time::sleep;

use crate::{auth::TgAuth, jwt::Jwt};

#[derive(Clone)]
pub struct ContextBuilder {
    ledger: Arc<Ledger>,
    bot: BotApp,
    jwt: Arc<Jwt>,
    auth: TgAuth,
}

impl ContextBuilder {
    pub fn new(ledger: Arc<Ledger>, bot: BotApp) -> Self {
        let jwt = Arc::new(Jwt::new(bot.env.jwt_secret()));
        let auth = TgAuth::new(bot.env.tg_token());
        ContextBuilder {
            ledger,
            bot,
            jwt,
            auth,
        }
    }

    pub async fn build(&self, tg_id: i64) -> Result<Context, Error> {
        let mut session = self.ledger.db.start_session().await?;

        let user = self.ledger.users.get_by_tg_id(&mut session, tg_id).await?;
        let mut user = if let Some(user) = user {
            user
        } else {
            sleep(std::time::Duration::from_secs(1)).await;
            return Err(eyre::eyre!("User not found"));
        };
        self.ledger.users.resolve_family(&mut session, &mut user).await?;

        session.set_actor(user.id);

        let state = self
            .bot
            .state
            .get_state(ChatId(user.tg_id))
            .unwrap_or_default();

        let origin = if let Some(origin) = state.origin {
            origin
        } else {
            Origin {
                chat_id: ChatId(user.tg_id),
                message_id: MessageId(0),
                tkn: ValidToken::new(),
            }
        };

        let tg_bot = TgBot::new(
            self.bot.bot.clone(),
            self.bot.state.tokens(),
            origin,
            self.bot.env.clone(),
        );
        Ok(Context::new(
            tg_bot,
            user,
            self.ledger.clone(),
            session,
            true,
        ))
    }
}

pub async fn middleware(
    State(state): State<ContextBuilder>,
    mut request: Request,
    next: Next,
) -> Response {
    if let Some(auth_header) = request.headers().get("Authorization") {
        if let Ok((auth_key, jwt)) = state.jwt.claims::<Claims>(auth_header) {
            match state.build(auth_key.id).await {
                Ok(ctx) => {
                    request.extensions_mut().insert(Arc::new(ctx));
                    request.extensions_mut().insert(jwt);
                }
                Err(err) => {
                    warn!("Failed to build context: {}", err);
                    sleep(std::time::Duration::from_secs(1)).await;
                    return (StatusCode::UNAUTHORIZED, "Unauthorized").into_response();
                }
            }
        } else {
            warn!("Invalid Authorization header:{:?}", request);
            sleep(std::time::Duration::from_secs(1)).await;
            return (StatusCode::UNAUTHORIZED, "Unauthorized").into_response();
        }
    } else {
        if let Ok(Query(params)) = Query::<BTreeMap<String, String>>::try_from_uri(request.uri()) {
            match state.auth.validate(params) {
                Ok(user_id) => match state.build(user_id).await {
                    Ok(ctx) => {
                        request.extensions_mut().insert(Arc::new(ctx));
                        match state.jwt.make_jwt(Claims::new(user_id)) {
                            Ok(jwt) => {
                                request.extensions_mut().insert(jwt);
                            }
                            Err(err) => {
                                warn!("Failed to make jwt: {}", err);
                                sleep(std::time::Duration::from_secs(1)).await;
                                return (StatusCode::UNAUTHORIZED, "Unauthorized").into_response();
                            }
                        }
                    }
                    Err(err) => {
                        warn!("Failed to build context: {}", err);
                        sleep(std::time::Duration::from_secs(1)).await;
                        return (StatusCode::UNAUTHORIZED, "Unauthorized").into_response();
                    }
                },
                Err(err) => {
                    warn!("Failed to validate tg auth: {}", err);
                    sleep(std::time::Duration::from_secs(1)).await;
                    return (StatusCode::UNAUTHORIZED, "Unauthorized").into_response();
                }
            }
        } else {
            warn!("No Authorization header:{:?}", request);
            sleep(std::time::Duration::from_secs(1)).await;
            return (StatusCode::UNAUTHORIZED, "Unauthorized").into_response();
        }
    }

    next.run(request).await
}

#[derive(Debug, Serialize, Deserialize)]
struct Claims {
    id: i64,
    exp: u64,
}

impl Claims {
    fn new(id: i64) -> Self {
        Claims {
            id,
            exp: (Utc::now() + Duration::days(7)).timestamp() as u64,
        }
    }
}
