use serde::{Serialize, Deserialize};
use super::token::Token;
use super::RoleTrait;

#[derive(Serialize, Deserialize)]
pub struct AnyRole;

impl RoleTrait for AnyRole {
    const ROLE: super::Role = super::Role::Customer;
    fn assert_role(_role: &super::Role) -> bool { true }
}

pub struct Auth<T = AnyRole> {
    user: super::Users,
    role: T,
}

impl<T> Auth<T> {
    pub fn role(&self) -> &T {
        &self.role
    }
}

impl<T> Auth<T> where T: serde::de::DeserializeOwned {
    pub fn from_token(token: Token) -> serde_json::Result<Self> {
        let result = serde_json::from_value(token.role_data)?;
        Ok(Self { user: token.user, role: result })
    }
}

impl<T> From<Auth<T>> for Token where T: Serialize {
    fn from(value: Auth<T>) -> Self {
        Token::new(value.user, serde_json::to_value(value.role).unwrap())
    }
}

impl<T> std::ops::Deref for Auth<T> {
    type Target = super::Users;

    fn deref(&self) -> &Self::Target {
        &self.user
    }
}

