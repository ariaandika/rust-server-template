use std::sync::Arc;
use axum::response::IntoResponse;
use axum::Extension;
use super::token::Token;
use super::{error, Admin, Auth};
use crate::libs::extractor::Json;
use crate::libs::error::{Result, Error};


#[derive(serde::Serialize, serde::Deserialize)]
pub struct Login {
    phone: String,
    password: String,
}

pub async fn login(
    Extension(db): Extension<Arc<sqlx::PgPool>>,
    Json(input): Json<Login>,
) -> Result<Json<serde_json::Value>> {
    use argon2::{Argon2, PasswordHash, PasswordVerifier};

    let Some(user) = sqlx::query_as::<_, super::Users>("SELECT * FROM users WHERE phone = $1")
        .bind(&input.phone).fetch_optional(&*db).await?
    else {
        return Err(error::invalid_credential());
    };

    let hash_passwd = user.password.clone();

    let pass_match = tokio::task::spawn_blocking(move||{
        let input_passwd = input.password.clone();
        let hash_passwd = hash_passwd;
        let ok = Argon2::default()
            .verify_password(
                input_passwd.as_bytes(),
                &PasswordHash::new(&hash_passwd).map_err(Error::fatal)?,
            ).is_ok();

        Ok::<_, Error>(ok)
    }).await.map_err(Error::fatal)??;

    if !pass_match {
        return Err(error::invalid_credential());
    }

    let role_data = user.create_role_data(&*db).await?;

    let token = Token::sign(user, role_data)?;

    Ok(Json(serde_json::json!({ "token": token })))
}

pub async fn login_cookie(
    db: Extension<Arc<sqlx::PgPool>>,
    data: Json<Login>
) -> Result {
    let Json(result) = login(db, data).await?;
    let cookie = if cfg!(debug_assertions) {
        format!("access_token={}; Path=/; HttpOnly;",result["token"])
    } else {
        format!("access_token={}; Path=/; HttpOnly; Secure; SameSite=None",result["token"])
    };
    Ok(([("set-cookie", cookie)], Json(result)).into_response())
}

pub async fn session(
    token: Option<Token>,
) -> Json<serde_json::Value> {
    if let Some(token) = token {
        Json(serde_json::to_value(token).unwrap())
    } else {
        Json(serde_json::Value::Null)
    }
}

pub async fn admin(
    auth: Auth<Admin>
) -> Json<Token> {
    let _role_data = auth.role();
    Json(auth.into())
}


