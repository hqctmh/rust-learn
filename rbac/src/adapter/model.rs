use axum::Json;
use serde::{Deserialize, Serialize};

use crate::adapter::error::AppError;

#[derive(Debug, Serialize)]
pub struct ApiResponse<T> {
    pub code: u16,
    pub message: String,
    pub data: Option<T>,
}

impl<T> ApiResponse<T> {
    pub fn success(data: T) -> Self {
        Self {
            code: 0,
            message: "success".to_string(),
            data: Some(data),
        }
    }
}

impl ApiResponse<()> {
    pub fn error(code: u16, message: impl Into<String>) -> Self {
        Self {
            code,
            message: message.into(),
            data: None,
        }
    }
}

pub type ApiResult<T> = Result<Json<ApiResponse<T>>, AppError>;

#[derive(Debug, Serialize, Deserialize)]
pub struct UserRegiestParam {
    pub username: String,
    pub password: String,
    pub display_name: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct UserLoginParam {
    pub username: String,
    pub password: String,
}
