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
}

crate::enum_encode!(Role);

