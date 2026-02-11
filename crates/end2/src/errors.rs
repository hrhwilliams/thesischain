use core::fmt;

use axum::{
    Json,
    http::StatusCode,
    response::{IntoResponse, Response},
};
use base64::DecodeError;
use serde::Serialize;
use tokio::task::JoinError;

#[derive(Debug, Serialize)]
pub struct ApiError {
    pub status: u16,
    pub message: String,
    pub detail: Option<String>,
}

impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        (
            StatusCode::from_u16(self.status).expect("status code must be in range [100, 999]"),
            Json(self),
        )
            .into_response()
    }
}

pub enum ExtractError {
    NoSession,
    NoUser,
    CookieError(String),
    InvalidSessionId(String),
    LookupError(AppError),
}

impl From<ExtractError> for ApiError {
    fn from(value: ExtractError) -> Self {
        match value {
            ExtractError::NoSession => Self {
                status: StatusCode::UNAUTHORIZED.into(),
                message: "missing session cookie".to_string(),
                detail: None,
            },
            ExtractError::NoUser => Self {
                status: StatusCode::UNAUTHORIZED.into(),
                message: "no such user".to_string(),
                detail: None,
            },
            ExtractError::CookieError(s) => Self {
                status: StatusCode::INTERNAL_SERVER_ERROR.into(),
                message: "error extracting cookie".to_string(),
                detail: Some(s),
            },
            ExtractError::InvalidSessionId(s) => Self {
                status: StatusCode::BAD_REQUEST.into(),
                message: "bad session token".to_string(),
                detail: Some(s),
            },
            ExtractError::LookupError(e) => e.into(),
        }
    }
}

impl From<RegistrationError> for ApiError {
    fn from(value: RegistrationError) -> Self {
        match value {
            RegistrationError::InvalidUsernameOrPassword => Self {
                status: StatusCode::BAD_REQUEST.into(),
                message: "bad username or password".to_string(),
                detail: None,
            },
            RegistrationError::PasswordMismatch => Self {
                status: StatusCode::BAD_REQUEST.into(),
                message: "passwords do not match".to_string(),
                detail: None,
            },
            RegistrationError::UsernameExists => Self {
                status: StatusCode::BAD_REQUEST.into(),
                message: "username taken".to_string(),
                detail: None,
            },
            RegistrationError::InvalidDiscordId(s) => Self {
                status: StatusCode::UNAUTHORIZED.into(),
                message: "invalid discord ID".to_string(),
                detail: Some(s),
            },
            RegistrationError::InternalError(e) => e.into(),
        }
    }
}

impl From<AppError> for RegistrationError {
    fn from(value: AppError) -> Self {
        Self::InternalError(value)
    }
}

impl From<AppError> for ApiError {
    fn from(value: AppError) -> Self {
        match value {
            AppError::ArgonError(s) => Self {
                status: StatusCode::INTERNAL_SERVER_ERROR.into(),
                message: "argon2 error".to_string(),
                detail: Some(s),
            },
            AppError::ChallengeFailed(s) => Self {
                status: StatusCode::BAD_REQUEST.into(),
                message: "challenge response failed".to_string(),
                detail: Some(s),
            },
            AppError::OAuth(s) => Self {
                status: StatusCode::BAD_REQUEST.into(),
                message: "oauth flow failed".to_string(),
                detail: Some(s),
            },
            AppError::InvalidB64(s) => Self {
                status: StatusCode::BAD_REQUEST.into(),
                message: "failed to decode base64 string".to_string(),
                detail: Some(s),
            },
            AppError::InvalidKey(s) => Self {
                status: StatusCode::INTERNAL_SERVER_ERROR.into(),
                message: "invalid verifying key".to_string(),
                detail: Some(s),
            },
            AppError::InvalidKeySize => Self {
                status: StatusCode::INTERNAL_SERVER_ERROR.into(),
                message: "verifying key has invalid size".to_string(),
                detail: None,
            },
            AppError::InvalidSignature => Self {
                status: StatusCode::BAD_REQUEST.into(),
                message: "signature was invalid".to_string(),
                detail: None,
            },
            AppError::NoSuchUser => Self {
                status: StatusCode::BAD_REQUEST.into(),
                message: "user does not exist".to_string(),
                detail: None,
            },
            AppError::UserError(s) => Self {
                status: StatusCode::BAD_REQUEST.into(),
                message: "bad input".to_string(),
                detail: Some(s),
            },
            AppError::PoolError(s) => Self {
                status: StatusCode::INTERNAL_SERVER_ERROR.into(),
                message: "database connection failed".to_string(),
                detail: Some(s),
            },
            AppError::QueryFailed(s) => Self {
                status: StatusCode::INTERNAL_SERVER_ERROR.into(),
                message: "query failed".to_string(),
                detail: Some(s),
            },
            AppError::Unauthorized => Self {
                status: StatusCode::UNAUTHORIZED.into(),
                message: "tried to perform an unauthorized action".to_string(),
                detail: None,
            },
            AppError::ValueError(s) => Self {
                status: StatusCode::INTERNAL_SERVER_ERROR.into(),
                message: "failed to convert a value".to_string(),
                detail: Some(s),
            },
        }
    }
}

impl fmt::Display for AppError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{self:?}")
    }
}

impl From<diesel::result::Error> for AppError {
    fn from(e: diesel::result::Error) -> Self {
        Self::QueryFailed(e.to_string())
    }
}

impl From<JoinError> for AppError {
    fn from(e: JoinError) -> Self {
        Self::PoolError(e.to_string())
    }
}

impl From<DecodeError> for AppError {
    fn from(e: DecodeError) -> Self {
        Self::InvalidB64(e.to_string())
    }
}

#[derive(Debug)]
pub enum AppError {
    ArgonError(String),
    ChallengeFailed(String),
    OAuth(String),
    InvalidB64(String),
    InvalidKey(String),
    InvalidKeySize,
    InvalidSignature,
    NoSuchUser,
    UserError(String),
    PoolError(String),
    QueryFailed(String),
    Unauthorized,
    ValueError(String),
}

#[derive(Debug)]
pub enum RegistrationError {
    InvalidUsernameOrPassword,
    PasswordMismatch,
    UsernameExists,
    InternalError(AppError),
    InvalidDiscordId(String),
}

pub enum LoginError {
    InternalError(AppError),
    InvalidDiscordId(String),
    InvalidPassword,
    NoSuchUser,
    NoPassword,
}

impl From<AppError> for LoginError {
    fn from(value: AppError) -> Self {
        Self::InternalError(value)
    }
}

impl From<LoginError> for ApiError {
    fn from(value: LoginError) -> Self {
        match value {
            LoginError::InternalError(e) => e.into(),
            LoginError::InvalidDiscordId(s) => Self {
                status: StatusCode::UNAUTHORIZED.into(),
                message: "invalid discord ID".to_string(),
                detail: Some(s),
            },
            LoginError::InvalidPassword | LoginError::NoSuchUser => Self {
                status: StatusCode::UNAUTHORIZED.into(),
                message: "invalid password or username".to_string(),
                detail: None,
            },
            LoginError::NoPassword => Self {
                status: StatusCode::UNAUTHORIZED.into(),
                message: "no password provided".to_string(),
                detail: None,
            },
        }
    }
}

#[derive(Clone, Debug)]
pub enum OAuthError {
    FailedToBuildClient(String),
    FailedToCreateAuthUrl,
    FailedToStoreAttempt,
    FailedToRetrieveAttempt,
    FailedToGetToken(String),
    FailedToMakeSession,
    FailedQuery,
    StateMismatch,
}

impl From<OAuthError> for AppError {
    fn from(value: OAuthError) -> Self {
        match value {
            OAuthError::FailedQuery => Self::OAuth("failed to query".to_string()),
            OAuthError::FailedToBuildClient(s) | OAuthError::FailedToGetToken(s) => Self::OAuth(s),
            OAuthError::FailedToCreateAuthUrl => Self::OAuth("failed to create url".to_string()),
            OAuthError::FailedToMakeSession => Self::OAuth("failed to make session".to_string()),
            OAuthError::FailedToRetrieveAttempt => {
                Self::OAuth("failed to retrieve challenge".to_string())
            }
            OAuthError::FailedToStoreAttempt => Self::OAuth("failed to store attempt".to_string()),
            OAuthError::StateMismatch => Self::OAuth("states did not match".to_string()),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn app_error_unauthorized_maps_to_401() {
        let api: ApiError = AppError::Unauthorized.into();
        assert_eq!(api.status, 401);
    }

    #[test]
    fn app_error_no_such_user_maps_to_400() {
        let api: ApiError = AppError::NoSuchUser.into();
        assert_eq!(api.status, 400);
    }

    #[test]
    fn app_error_query_failed_maps_to_500() {
        let api: ApiError = AppError::QueryFailed("test".into()).into();
        assert_eq!(api.status, 500);
    }

    #[test]
    fn login_error_invalid_password_maps_to_401() {
        let api: ApiError = LoginError::InvalidPassword.into();
        assert_eq!(api.status, 401);
        assert_eq!(api.message, "invalid password or username");
    }

    #[test]
    fn login_error_no_such_user_same_as_invalid_password() {
        let api: ApiError = LoginError::NoSuchUser.into();
        assert_eq!(api.status, 401);
        assert_eq!(api.message, "invalid password or username");
    }

    #[test]
    fn registration_error_password_mismatch_maps_to_400() {
        let api: ApiError = RegistrationError::PasswordMismatch.into();
        assert_eq!(api.status, 400);
    }

    #[test]
    fn extract_error_no_session_maps_to_401() {
        let api: ApiError = ExtractError::NoSession.into();
        assert_eq!(api.status, 401);
    }
}
