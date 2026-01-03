use axum::{
    Json,
    http::StatusCode,
    response::{IntoResponse, Response},
};
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

impl From<RegistrationError> for ApiError {
    fn from(value: RegistrationError) -> Self {
        match value {
            RegistrationError::InvalidUsername => ApiError {
                status: StatusCode::BAD_REQUEST.into(),
                message: "bad username".to_string(),
                detail: None,
            },
            RegistrationError::UsernameExists => ApiError {
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
            AppError::ChallengeFailed(s) => ApiError {
                status: StatusCode::BAD_REQUEST.into(),
                message: "challenge response failed".to_string(),
                detail: Some(s),
            },
            AppError::InvalidB64 => ApiError {
                status: StatusCode::BAD_REQUEST.into(),
                message: "failed to decode base64 string".to_string(),
                detail: None,
            },
            AppError::InvalidKey(s) => ApiError {
                status: StatusCode::INTERNAL_SERVER_ERROR.into(),
                message: "invalid verifying key".to_string(),
                detail: Some(s),
            },
            AppError::InvalidKeySize => ApiError {
                status: StatusCode::INTERNAL_SERVER_ERROR.into(),
                message: "verifying key has invalid size".to_string(),
                detail: None,
            },
            AppError::InvalidSignature => ApiError {
                status: StatusCode::BAD_REQUEST.into(),
                message: "signature was invalid".to_string(),
                detail: None,
            },
            AppError::NoSuchUser => ApiError {
                status: StatusCode::BAD_REQUEST.into(),
                message: "user does not exist".to_string(),
                detail: None,
            },
            AppError::PoolError(s) => ApiError {
                status: StatusCode::INTERNAL_SERVER_ERROR.into(),
                message: "database connection failed".to_string(),
                detail: Some(s),
            },
            AppError::QueryFailed(s) => ApiError {
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

pub enum InputError {}

pub enum AppError {
    ChallengeFailed(String),
    InvalidB64,
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
