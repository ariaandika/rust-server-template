use axum::{extract::FromRequestParts, http::request::Parts};
use super::{auth::Auth, RoleTrait};
use crate::libs::error::{Error, Result};
use super::error;

#[axum::async_trait]
impl<T,S> FromRequestParts<S> for Auth<T> where S: Send + Sync, T: RoleTrait + serde::de::DeserializeOwned {
    type Rejection = Error;

    async fn from_request_parts(parts: &mut Parts, _state: &S) -> Result<Self> {
        let token = super::token::Token::from_request_parts(parts, _state).await?;

        if T::assert_role(&token.user.role) {
            if let Ok(auth) = Self::from_token(token) {
                Ok(auth)
            } else {
                Err(error::token_invalid())
            }
        } else {
            Err(error::unauthorized())
        }
    }
}

