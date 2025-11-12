use axum::{
    Json,
    http::StatusCode,
    response::{IntoResponse, Response},
};
use serde::Serialize;

pub enum ApiError {
    Generic,
    KademliaError(String),
}

#[derive(Serialize)]
struct ErrorJson {
    message: String,
    detail: Option<String>
}

impl From<ApiError> for Response {
    fn from(value: ApiError) -> Self {
        match value {
            ApiError::Generic => (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ErrorJson {
                    message: "generic server error".to_string(),
                    detail: None
                }),
            )
                .into_response(),
            ApiError::KademliaError(reason) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ErrorJson {
                    message: "kademlia error".to_string(),
                    detail: Some(reason)
                }),
            )
                .into_response(),
        }
    }
}

impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        self.into()
    }
}
