use axum::{extract::FromRequestParts, http::request::Parts};
use chrono::Utc;
use serde::{Deserialize, Serialize};
use jsonwebtoken as j;
use super::Users;
use super::error;
use crate::libs::error::{Result, Error};


lazy_static::lazy_static! {
    static ref SECRET: Vec<u8> = {
        let foo = dotenvy::var("JWT_SECRET").expect("reading JWT_SECRET env");
        foo.as_bytes().to_vec()
    };
    static ref VALIDATION: j::Validation = j::Validation::default();
    static ref ENCODING_KEY: j::EncodingKey = j::EncodingKey::from_secret(&SECRET);
    static ref DECODING_KEY: j::DecodingKey = j::DecodingKey::from_secret(&SECRET);
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Token {
    pub exp: usize,
    #[serde(flatten)]
    pub user: super::Users,
    pub role_data: serde_json::Value
}

impl Token {
    pub fn new(user: Users, role_data: serde_json::Value) -> Self {
        Self {
            user, role_data,
            exp: Utc::now()
                .checked_add_signed(chrono::Duration::days(7))
                .expect("valid timestamp")
                .timestamp() as usize,
        }
    }

    pub fn sign(user: Users, role_data: serde_json::Value) -> Result<String> {
        j::encode(&j::Header::default(), &Self::new(user, role_data), &ENCODING_KEY).map_err(Error::fatal)
    }

    pub fn verify(token: String) -> Result<Self> {
        j::decode::<Self>(&token, &DECODING_KEY, &VALIDATION).map(|e|e.claims).map_err(Error::from)
    }
}

fn take_cookie(parts: &axum::http::request::Parts) -> Option<String> {
    Some(parts.headers.get("cookie")?
        .to_str().ok()?
        .split("; ")
        .find(|e|e.starts_with("access_token="))?
        .replacen("access_token=","",1))
}

fn take_bearer(parts: &axum::http::request::Parts) -> Option<String> {
    Some(parts.headers.get("authorization")?
        .to_str().ok()?
        .split_once(" ")?
        .1.to_string())
}

#[axum::async_trait]
impl<S> FromRequestParts<S> for Token where S: Send + Sync {
    type Rejection = Error;

    async fn from_request_parts(parts: &mut Parts, _state: &S) -> Result<Self> {
        let token = {
            if let Some(token) = take_cookie(parts) { token }
            else if let Some(token) = take_bearer(parts) { token }
            else { return Err(error::unauthenticated()) }
        };

        Self::verify(token)
    }
}

impl From<jsonwebtoken::errors::Error> for Error {
    fn from(value: jsonwebtoken::errors::Error) -> Self {
        use jsonwebtoken::errors::ErrorKind::*;
        match value.kind() {
            InvalidToken | InvalidSignature | InvalidEcdsaKey | InvalidRsaKey(_) |
                MissingRequiredClaim(_) | Base64(_) | Json(_) |
                InvalidIssuer | MissingAlgorithm | InvalidAlgorithm | InvalidAudience |
                InvalidSubject | ImmatureSignature
                    => error::token_invalid(),

                ExpiredSignature
                    => error::token_expired(),

                RsaFailedSigning | InvalidAlgorithmName | InvalidKeyFormat | Utf8(_) | Crypto(_) | _
                    => Error::fatal(value),
        }
    }
}

