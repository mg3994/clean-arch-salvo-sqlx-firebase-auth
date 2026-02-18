use thiserror::Error;

#[derive(Error, Debug)]
pub enum AppError {
    #[error("not found: `{0}`")]
    NotFound(String),
    
    #[error("unauthorized: `{0}`")]
    Unauthorized(String),
    
    #[error("forbidden: `{0}`")]
    Forbidden(String),
    
    #[error("bad request: `{0}`")]
    BadRequest(String),

    #[error("conflict: `{0}`")]
    Conflict(String),

    #[error("validation error: `{0}`")]
    Validation(String),

    #[error("internal error: `{0}`")]
    Internal(String),
}

impl AppError {
    pub fn not_found<S: Into<String>>(msg: S) -> Self {
        Self::NotFound(msg.into())
    }

    pub fn unauthorized<S: Into<String>>(msg: S) -> Self {
        Self::Unauthorized(msg.into())
    }

    pub fn forbidden<S: Into<String>>(msg: S) -> Self {
        Self::Forbidden(msg.into())
    }

    pub fn bad_request<S: Into<String>>(msg: S) -> Self {
        Self::BadRequest(msg.into())
    }

    pub fn conflict<S: Into<String>>(msg: S) -> Self {
        Self::Conflict(msg.into())
    }

    pub fn validation<S: Into<String>>(msg: S) -> Self {
        Self::Validation(msg.into())
    }

    pub fn internal<S: Into<String>>(msg: S) -> Self {
        Self::Internal(msg.into())
    }
}

pub type AppResult<T> = Result<T, AppError>;
