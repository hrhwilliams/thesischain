use argon2::{Argon2, PasswordHasher, password_hash::SaltString};
use diesel::prelude::*;
use rand_core::OsRng;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::{AppError, RegistrationError, is_valid_username};

#[derive(Debug, Queryable, Selectable)]
#[diesel(table_name = crate::schema::user)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct User {
    pub id: Uuid,
    pub username: String,
    pub nickname: Option<String>,
    pub password: Option<String>,
}

#[derive(Debug, Deserialize, Insertable)]
#[diesel(table_name = crate::schema::user)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct NewUser {
    pub username: String,
    pub password: Option<String>,
}

#[derive(Deserialize)]
pub struct InboundUser {
    pub username: String,
    pub password: String,
    pub confirm_password: String,
}

impl TryFrom<InboundUser> for NewUser {
    type Error = RegistrationError;

    fn try_from(inbound: InboundUser) -> Result<Self, RegistrationError> {
        if !is_valid_username(&inbound.username)
        {
            return Err(RegistrationError::InvalidUsernameOrPassword);
        }
        if inbound.password != inbound.confirm_password {
            return Err(RegistrationError::PasswordMismatch);
        }

        let salt = SaltString::generate(&mut OsRng);

        let hash = Argon2::default()
            .hash_password(inbound.password.as_bytes(), &salt)
            .map_err(|e| RegistrationError::from(AppError::ArgonError(e.to_string())))?
            .to_string();

        Ok(Self {
            username: inbound.username.trim().to_string(),
            password: Some(hash),
        })
    }
}

#[derive(Serialize)]
pub struct OutboundUser {
    pub id: Uuid,
    pub username: String,
    pub nickname: Option<String>,
}

impl From<User> for OutboundUser {
    fn from(user: User) -> Self {
        Self {
            id: user.id,
            username: user.username,
            nickname: user.nickname,
        }
    }
}
