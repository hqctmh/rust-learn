use axum::{Json, http::StatusCode, response::IntoResponse};

use crate::adapter::model::ApiResponse;

pub enum AppError {
    BadRequest(String),
    Unauthorized,
    NotFound(String),
    Internal(anyhow::Error),
}

impl IntoResponse for AppError {
    fn into_response(self) -> axum::response::Response {
        let (status, message) = match self {
            AppError::BadRequest(message) => (StatusCode::BAD_REQUEST, message),

            AppError::Unauthorized => (StatusCode::UNAUTHORIZED, "未登录或登录已失效".to_string()),

            AppError::NotFound(message) => (StatusCode::NOT_FOUND, message),

            AppError::Internal(error) => {
                // 正式项目建议使用 tracing
                eprintln!("服务器内部错误：{error:?}");

                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    "服务器内部错误".to_string(),
                )
            }
        };

        let body = ApiResponse::<()>::error(status.as_u16(), message);

        (status, Json(body)).into_response()
    }
}

impl From<anyhow::Error> for AppError {
    fn from(error: anyhow::Error) -> Self {
        AppError::Internal(error)
    }
}
