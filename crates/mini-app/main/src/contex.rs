use axum::{
    extract::{Query, Request, State},
    http::StatusCode,
    middleware::Next,
    response::{IntoResponse as _, Response},
};
use axum_extra::extract::{
    cookie::{self, Cookie},
    CookieJar,
};
use bot_core::{
    bot::{Origin, TgBot, ValidToken},
    context::Context,
};
use bot_main::BotApp;
use eyre::Error;
use ledger::Ledger;
use log::warn;
use serde::Deserialize;
use std::{env, sync::Arc};
use teloxide::types::{ChatId, MessageId};
use tokio::time::sleep;

#[derive(Clone)]
pub struct ContextBuilder {
    ledger: Ledger,
    bot: BotApp,
}

impl ContextBuilder {
    pub fn new(ledger: Ledger, bot: BotApp) -> Self {
        ContextBuilder { ledger, bot }
    }

    pub async fn build(&self, auth_key: &str) -> Result<Context, Error> {
        let mut session = self.ledger.db.start_session().await?;

        let user = self.ledger.auth.auth(&mut session, auth_key).await;
        let user = if let Ok(user) = user {
            user
        } else {
            sleep(std::time::Duration::from_secs(1)).await;
            return Err(eyre::eyre!("User not found"));
        };

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
    let mut set_cookie = None;
    CookieJar::from_headers(request.headers()).get("auth");
    let ctx = if let Some(key) = CookieJar::from_headers(request.headers()).get("auth") {
        let auth_key = key.value();
        state.build(auth_key).await
    } else {
        match Query::<Key>::try_from_uri(request.uri()) {
            Ok(key) => {
                let key = key.0;
                if let Ok(app_key) = env::var("MINI_APP_KEY") {
                    if key.app_key != app_key {
                        warn!("Unauthorized access attempt with key: {}", app_key);
                        sleep(std::time::Duration::from_secs(1)).await;
                        return (StatusCode::UNAUTHORIZED, "Unauthorized").into_response();
                    }
                } else {
                    warn!("MINI_APP_KEY not set");
                    sleep(std::time::Duration::from_secs(1)).await;
                    return (StatusCode::UNAUTHORIZED, "Unauthorized").into_response();
                };
                let ctx = state.build(&key.user_key).await;
                set_cookie = Some(key.user_key);
                ctx
            }
            Err(err) => {
                warn!("Failed to parse key: {}", err);
                sleep(std::time::Duration::from_secs(1)).await;
                return (StatusCode::UNAUTHORIZED, "Unauthorized").into_response();
            }
        }
    };

    let ctx = match ctx {
        Ok(ctx) => ctx,
        Err(err) => {
            warn!("Failed to build context: {}", err);
            sleep(std::time::Duration::from_secs(1)).await;
            return (StatusCode::UNAUTHORIZED, "Unauthorized").into_response();
        }
    };

    request.extensions_mut().insert(Arc::new(ctx));
    let mut response = next.run(request).await;
    if let Some(auth_key) = set_cookie {
        let cookie = Cookie::build(("auth", auth_key))
            .http_only(true)
            .secure(true)
            .path("/")
            .domain(std::env::var("COOKIE_DOMAIN").unwrap_or_default())
            .same_site(cookie::SameSite::Strict)
            .build();
        response
            .headers_mut()
            .insert("Set-Cookie", cookie.to_string().parse().unwrap());
    }

    response
}

#[derive(Deserialize)]
pub struct Key {
    user_key: String,
    app_key: String,
}
