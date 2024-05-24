use std::sync::Arc;
use axum::extract::Query;
use axum::response::IntoResponse;
use axum::Extension;
use super::token::Token;
use super::{error, Admin, Auth, Users};
use crate::libs::extractor::Json;
use crate::libs::error::{Result, Error};


pub async fn session(token: Option<Token>) -> Json<serde_json::Value> {
    if let Some(token) = token {
        Json(serde_json::to_value(token).unwrap())
    } else {
        Json(serde_json::Value::Null)
    }
}

pub async fn admin(auth: Auth<Admin>) -> Json<Token> {
    let _role_data = auth.role();
    Json(auth.into())
}

#[derive(serde::Deserialize)]
pub struct Login {
    phone: String,
    password: String,
}

#[derive(serde::Deserialize)]
pub struct LoginQuery {
    as_cookie: Option<String>
}

pub async fn login_handler(
    db: Extension<Arc<sqlx::PgPool>>,
    Query(query): Query<LoginQuery>,
    input: Json<Login>,
) -> Result {
    let Json(res) = login(db, input).await?;

    if let Some(_) = query.as_cookie {
        Ok(login_cookie(res))
    } else {
        Ok(Json(res).into_response())
    }
}

pub fn login_cookie(result: serde_json::Value) -> axum::response::Response {
    let cookie = if cfg!(debug_assertions) {
        format!("access_token={}; Path=/; HttpOnly;",result["token"])
    } else {
        format!("access_token={}; Path=/; HttpOnly; Secure; SameSite=None",result["token"])
    };
    ([("set-cookie", cookie)], Json(result)).into_response()
}

const DUMMY_PASSWD: &'static str = "$argon2id$v=19$m=19456,t=2,p=1$rQNTQqpg6Sk+edlS31AM0A$mHEeiZOfvaI8leYwNX9o22dZxX2wSnm1rpbDN9EF9PU";

pub async fn login(
    Extension(db): Extension<Arc<sqlx::PgPool>>,
    Json(input): Json<Login>,
) -> Result<Json<serde_json::Value>> {
    use argon2::{Argon2, PasswordHash, PasswordVerifier};

    let user = sqlx::query_as::<_, Users>("SELECT * FROM users WHERE phone = $1")
        .bind(&input.phone).fetch_optional(&*db).await?;

    let (hash_passwd,user) = if let Some(user) = user {
        (user.password.clone(), Some(user))
    } else {
        (DUMMY_PASSWD.to_string(), None)
    };

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

    let Some(user) = user else { return Err(error::invalid_credential()) };

    let role_data = user.create_role_data(&*db).await?;

    let token = Token::sign(user, role_data)?;

    Ok(Json(serde_json::json!({ "token": token })))
}


#[derive(serde::Deserialize)]
pub struct Register {
    pub name: String,
    pub phone: String,
    pub password: String,
}

pub async fn register(
    Extension(db): Extension<Arc<sqlx::PgPool>>,
    Json(input): Json<Register>,
) -> Result<Json<Users>> {

    // TODO: Duplicate Check

    if let Some((name,phone)) = sqlx::query_as::<_, (String,String)>(
        "SELECT name,phone FROM users WHERE name = $1 OR phone = $2")
        .bind(&input.name)
        .bind(&input.phone)
        .fetch_optional(&*db).await?
    {
        let msg = if name == input.name {
            if phone == input.phone {
                "name and phone"
            } else {
                "name"
            }
        } else if phone == input.phone {
            "phone"
        } else {
            unreachable!()
        };

        use axum::http::StatusCode;

        return Err(Error::from_status(StatusCode::CONFLICT, format!("{msg} already used") ));
    }

    let hashed = tokio::spawn(async move {
        use argon2::password_hash::{rand_core::OsRng, PasswordHasher, SaltString};
        let input_passwd = input.password.clone();
        let hashed = argon2::Argon2::default().hash_password(
            input_passwd.as_bytes(),
            &SaltString::generate(&mut OsRng))
            .map_err(Error::fatal)?
            .to_string();

        Ok::<_, Error>(hashed)
    }).await.map_err(Error::fatal)??;

    let user = sqlx::query_as(
    "INSERT INTO users (name,phone,password,role,metadata) VALUES ($1,$2,$3,$4,'{}'::json) RETURNING *")
        .bind(&input.name)
        .bind(&input.phone)
        .bind(&hashed)
        .bind(&super::Role::default().to_string())
        .fetch_one(&*db).await?;

    Ok(Json(user))
}


