use axum::http::HeaderValue;
use eyre::{eyre, Error};
use jsonwebtoken::{DecodingKey, EncodingKey, Header, Validation};
use serde::{de::DeserializeOwned, Deserialize, Serialize};

pub struct Jwt {
    jwt_decode: DecodingKey,
    jwt_encode: EncodingKey,
    validation: Validation,
    header: Header,
}

impl Jwt {
    pub fn new(secret: &str) -> Self {
        let jwt_decode = DecodingKey::from_secret(secret.as_bytes());
        let jwt_encode = EncodingKey::from_secret(secret.as_bytes());
        let validation = Validation::default();
        let header = Header::default();
        Jwt {
            jwt_decode,
            jwt_encode,
            validation,
            header,
        }
    }

    pub fn claims<C: DeserializeOwned>(
        &self,
        header: &HeaderValue,
    ) -> Result<(C, JwtToken), Error> {
        let auth_key = header.to_str()?;
        let jwt = auth_key
            .strip_prefix("Bearer ")
            .ok_or_else(|| eyre!("No Bearer"))?;
        let token = jsonwebtoken::decode::<C>(jwt, &self.jwt_decode, &self.validation)?;
        Ok((
            token.claims,
            JwtToken {
                key: jwt.to_string(),
            },
        ))
    }

    pub fn make_jwt<C: Serialize>(&self, claims: C) -> Result<JwtToken, Error> {
        let key = jsonwebtoken::encode(&self.header, &claims, &self.jwt_encode)?;
        Ok(JwtToken { key })
    }
}

#[derive(Clone, Serialize, Deserialize)]
pub struct JwtToken {
    key: String,
}
