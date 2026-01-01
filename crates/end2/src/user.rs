use argon2::password_hash::PasswordHashString;
use serde::Deserialize;
use std::fmt;
use uuid::Uuid;

#[derive(Clone, Debug, Deserialize, Hash, PartialEq, Eq)]
pub struct UserName(String);

impl UserName {
    pub fn len(&self) -> usize {
        self.0.len()
    }
}

impl From<String> for UserName {
    fn from(value: String) -> Self {
        Self(value)
    }
}

impl fmt::Display for UserName {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        // Adjust 'self.0' if your struct has named fields (e.g., self.username)
        write!(f, "{}", self.0)
    }
}

#[derive(Clone, Debug)]
pub struct UserInfo {
    pub id: Uuid,
    pub username: UserName,
    pub password: PasswordHashString,
}
