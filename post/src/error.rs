use thiserror::Error;

#[derive(Debug, Error)]
pub enum ForumError {
    #[error("请先登录")]
    Unauthorized,
    #[error("没有权限执行该操作")]
    Forbidden,
    #[error("资源不存在")]
    NotFound,
    #[error("请求冲突: {0}")]
    Conflict(String),
    #[error("请求数据不合法: {0}")]
    Validation(String),
    #[error("服务内部错误")]
    Internal,
}
