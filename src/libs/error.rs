use axum::{extract::Json, http::StatusCode, response::{IntoResponse, Response}};

#[derive(Debug)]
pub struct Error(pub Response);

pub type Result<T = Response> = std::result::Result<T, Error>;

impl Error {
    pub fn fatal<T>(value: T) -> Self where T: std::fmt::Display {
        eprintln!("{value}");
        Self((
            StatusCode::INTERNAL_SERVER_ERROR,
            Self::msg("INTERNAL_SERVER_ERROR", "there is a problem with the server")
        ).into_response())
    }
    pub fn msg<M>(error: M, message: M) -> Json<serde_json::Value> where M: serde::Serialize {
        Json(serde_json::json!({ "error": error, "message": message }))
    }
}

impl axum::response::IntoResponse for Error {
    fn into_response(self) -> Response {
        self.0
    }
}

impl From<Response> for Error {
    fn from(value: Response) -> Self {
        Self(value)
    }
}

impl From<sqlx::Error> for Error {
    fn from(value: sqlx::Error) -> Self {
        use sqlx::error::DatabaseError;

        let Some(constraint) = value.as_database_error().and_then(DatabaseError::constraint) else {
            return Self::fatal(value);
        };

        let true = constraint.ends_with("_key") else {
            return Self::fatal(value);
        };

        let _ct = constraint.split("_").take(2).collect::<Vec<&str>>();

        Self::fatal(value)
    }
}



