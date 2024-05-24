use axum::http::StatusCode;

use crate::libs::error::Error;

pub fn token_invalid() -> Error {
    Error::new(StatusCode::UNAUTHORIZED, "TOKEN_INVALID", "user data may have changed, please login again".into())
}

pub fn token_expired() -> Error {
    Error::new(StatusCode::UNAUTHORIZED, "TOKEN_EXPIRED", "session expired, please login again".into())
}

pub fn invalid_credential() -> Error {
    Error::new(StatusCode::UNPROCESSABLE_ENTITY, "INVALID_CREDENTIAL", "phone or password invalid".into())
}

pub fn unauthorized() -> Error {
    Error::from_status(StatusCode::UNAUTHORIZED, "you are not allowed to access this resource".into())
}

pub fn unauthenticated() -> Error {
    Error::from_status(StatusCode::UNAUTHORIZED, "plase login before continue".into())
}

