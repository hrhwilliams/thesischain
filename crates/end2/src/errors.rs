use std::f64::consts::E;

use axum::{
    Json,
    http::StatusCode,
    response::{IntoResponse, Response},
};
use base64::DecodeError;
use serde::Serialize;

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
    CookieError(String),
    InvalidSessionId(String),
    LookupError(AppError),
}

impl From<ExtractError> for ApiError {
    fn from(value: ExtractError) -> Self {
        match value {
            ExtractError::NoSession => ApiError {
                status: StatusCode::UNAUTHORIZED.into(),
                message: "missing session cookie".to_string(),
                detail: None,
            },
            ExtractError::CookieError(s) => ApiError {
                status: StatusCode::INTERNAL_SERVER_ERROR.into(),
                message: "error extracting cookie".to_string(),
                detail: Some(s),
            },
            ExtractError::InvalidSessionId(s) => ApiError {
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
            RegistrationError::InvalidUsername => Self {
                status: StatusCode::BAD_REQUEST.into(),
                message: "bad username".to_string(),
                detail: None,
            },
            RegistrationError::UsernameExists => Self {
                status: StatusCode::BAD_REQUEST.into(),
                message: "username taken".to_string(),
                detail: None,
            },
            RegistrationError::System(e) => e.into(),
        }
    }
}

impl From<AppError> for RegistrationError {
    fn from(value: AppError) -> Self {
        Self::System(value)
    }
}

impl From<AppError> for ApiError {
    fn from(value: AppError) -> Self {
        match value {
            AppError::ChallengeFailed(s) => Self {
                status: StatusCode::BAD_REQUEST.into(),
                message: "challenge response failed".to_string(),
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
        }
    }
}

impl From<diesel::result::Error> for AppError {
    fn from(e: diesel::result::Error) -> Self {
        Self::QueryFailed(e.to_string())
    }
}

impl From<DecodeError> for AppError {
    fn from(e: DecodeError) -> Self {
        Self::InvalidB64(e.to_string())
    }
}

pub enum InputError {}

pub enum AppError {
    ChallengeFailed(String),
    InvalidB64(String),
    InvalidKey(String),
    InvalidKeySize,
    InvalidSignature,
    NoSuchUser,
    PoolError(String),
    QueryFailed(String),
}

pub enum RegistrationError {
    InvalidUsername,
    UsernameExists,
    System(AppError),
}
