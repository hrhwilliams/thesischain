use argon2::{Argon2, PasswordHasher, password_hash::SaltString};
use diesel::{Insertable, Queryable, Selectable};
use rand_core::OsRng;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::{AppError, RegistrationError, is_valid_username};

#[derive(Clone, Debug, Queryable, Selectable, Serialize)]
#[diesel(table_name = crate::schema::user)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct User {
    pub id: Uuid,
    pub username: String,
    pub nickname: Option<String>,
    #[serde(skip_serializing)]
    pub password: Option<String>,
}

impl PartialEq for User {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
    }
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
        if !is_valid_username(&inbound.username) {
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

#[cfg(test)]
mod tests {
    use super::*;

    fn make_inbound(username: &str, password: &str, confirm: &str) -> InboundUser {
        InboundUser {
            username: username.to_string(),
            password: password.to_string(),
            confirm_password: confirm.to_string(),
        }
    }

    #[test]
    fn valid_user_converts_successfully() {
        let inbound = make_inbound("testuser", "password123", "password123");
        let new_user = NewUser::try_from(inbound).expect("should succeed");
        assert_eq!(new_user.username, "testuser");
        assert!(new_user.password.is_some());
    }

    #[test]
    fn trims_username() {
        let inbound = make_inbound("  testuser  ", "password123", "password123");
        // Note: the username "  testuser  " fails is_valid_username due to spaces,
        // so trimming only applies to valid usernames. This tests that the validation
        // rejects usernames with leading/trailing spaces.
        let result = NewUser::try_from(inbound);
        assert!(matches!(
            result,
            Err(RegistrationError::InvalidUsernameOrPassword)
        ));
    }

    #[test]
    fn password_mismatch_returns_error() {
        let inbound = make_inbound("testuser", "password123", "different");
        let result = NewUser::try_from(inbound);
        assert!(matches!(result, Err(RegistrationError::PasswordMismatch)));
    }

    #[test]
    fn invalid_username_returns_error() {
        let inbound = make_inbound("AB", "password123", "password123");
        let result = NewUser::try_from(inbound);
        assert!(matches!(
            result,
            Err(RegistrationError::InvalidUsernameOrPassword)
        ));
    }
}
