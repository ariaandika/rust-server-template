use std::str::FromStr;
use serde::{Serialize, Deserialize};
use sqlx::FromRow;
use strum_macros::{Display as EnumDisplay, EnumString};
use serde_json::Value;
use chrono::{DateTime, Utc};

#[derive(Debug, Clone, Serialize, Deserialize, FromRow, PartialEq)]
pub struct Users {
    pub user_id: i32,
    pub name: String,
    pub phone: String,
    #[serde(default,skip_serializing)]
    pub password: String,
    pub role: Role,
    pub metadata: Value,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub verified_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Default, Clone, Serialize, Deserialize, EnumDisplay, EnumString, PartialEq)]
pub enum Role {
    Admin,
    #[default]
    Customer,
    Sales,
}

crate::enum_encode!(Role);



// IMPORTANT to derive serde here,
// or you will get confusing axum handler error,
// we need derive to parse session role data

#[derive(Serialize, Deserialize)]
pub struct Admin;
#[derive(Serialize, Deserialize)]
pub struct Customer;

#[derive(Serialize, Deserialize, sqlx::FromRow)]
pub struct Sales {
    pub wh_id: i32,
    pub wh_name: String,
}


pub trait RoleTrait {
    const ROLE: Role;

    fn assert_role(role: &Role) -> bool {
        role == &Self::ROLE
    }
}

impl RoleTrait for Admin {
    const ROLE: Role = Role::Admin;
}

impl RoleTrait for Customer {
    const ROLE: Role = Role::Customer;
}

impl RoleTrait for Sales {
    const ROLE: Role = Role::Sales;
}

impl Users {
    pub async fn create_role_data(&self, db: &sqlx::PgPool) -> crate::libs::error::Result<Value> {
        match self.role {
            Role::Admin => Ok(Value::Null),
            Role::Customer => Ok(Value::Null),
            Role::Sales => {
                let Some(sales) = sqlx::query_as::<_, Sales>(
                "SELECT w.wh_id,w.name as wh_name FROM employees e LEFT JOIN warehouses w ON w.wh_id = e.wh_id WHERE user_id = $1")
                    .bind(self.user_id).fetch_optional(db).await?
                else {
                    return Ok(Value::Null);
                };
                serde_json::to_value(&sales).map_err(crate::libs::error::Error::fatal)
            },
        }
    }
}


pub use auth::Auth;

pub mod error;
mod token;
mod auth;
mod traits;
pub mod query;

