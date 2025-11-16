use std::fmt;

use axum::{
    Json,
    http::StatusCode,
    response::{IntoResponse, Response},
};
use serde::Serialize;
use tracing::{error, warn};
use uuid::Uuid;

#[derive(Debug, thiserror::Error)]
pub enum ErrorType {
    #[error("Database error: {0}")]
    Database(String),

    //auth not implemented yet
    #[error("Authentication failed: {0}")]
    Authentication(String),
    #[error("Authorization failed: {0}")]
    Authorization(String),

    #[error("Validation error: {0}")]
    Validation(String),

    #[error("Not found: {0}")]
    NotFound(String),

    #[error("Configuration error: {0}")]
    Configuration(String),

    #[error("Internal server error: {0}")]
    Internal(String),
}

impl ErrorType {
    /// Returns the HTTP status code for this error type
    fn status_code(&self) -> StatusCode {
        match self {
            ErrorType::Database(_) => StatusCode::INTERNAL_SERVER_ERROR,
            ErrorType::Authentication(_) => StatusCode::UNAUTHORIZED,
            ErrorType::Authorization(_) => StatusCode::FORBIDDEN,
            ErrorType::Validation(_) => StatusCode::BAD_REQUEST,
            ErrorType::NotFound(_) => StatusCode::NOT_FOUND,
            ErrorType::Configuration(_) => StatusCode::INTERNAL_SERVER_ERROR,
            ErrorType::Internal(_) => StatusCode::INTERNAL_SERVER_ERROR,
        }
    }

    /// Returns whether this error should be logged at ERROR level (vs WARN)
    fn is_severe(&self) -> bool {
        matches!(
            self,
            ErrorType::Database(_) | ErrorType::Configuration(_) | ErrorType::Internal(_)
        )
    }
}

pub struct AppError {
    pub error_id: Uuid,

    pub error_type: ErrorType,

    pub source: anyhow::Error,

    pub context: Vec<String>,
}

impl AppError {
    pub fn new(error_type: ErrorType, source: anyhow::Error) -> Self {
        Self {
            error_id: Uuid::new_v4(),
            error_type,
            source,
            context: Vec::new(),
        }
    }

    pub fn with_context<S: Into<String>>(mut self, ctx: S) -> Self {
        self.context.push(ctx.into());
        self
    }

    pub fn log_error(&self) {
        let error_chain = format_traceback(&self.source);
        let context_str = if self.context.is_empty() {
            "No additional context".to_string()
        } else {
            self.context.join(" -> ")
        };

        if self.error_type.is_severe() {
            error!(
                error_id = %self.error_id,
                error_type = %self.error_type,
                status_code = %self.error_type.status_code(),
                context = %context_str,
                error_chain = %error_chain,
                "Application error occurred"
            );
        } else {
            warn!(
                error_id = %self.error_id,
                error_type = %self.error_type,
                status_code = %self.error_type.status_code(),
                context = %context_str,
                error_chain = %error_chain,
                "Application warning occurred"
            );
        }
    }
}

impl fmt::Debug for AppError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("AppError")
            .field("error_id", &self.error_id)
            .field("error_type", &self.error_type)
            .field("context", &self.context)
            .field("source", &self.source)
            .finish()
    }
}

impl fmt::Display for AppError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "[{}] {}: {}",
            self.error_id, self.error_type, self.source
        )
    }
}

fn format_traceback(error: &anyhow::Error) -> String {
    let mut chain = vec![error.to_string()];
    let mut source = error.source();

    while let Some(err) = source {
        chain.push(err.to_string());
        source = err.source();
    }

    chain.join(" | by: ")
}

#[derive(Serialize)]
struct ErrorResponse {
    error_id: String,
    message: String,
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        self.log_error();

        let status_code = self.error_type.status_code();

        let error_response = ErrorResponse {
            error_id: self.error_id.to_string(),
            message: self.error_type.to_string(),
        };

        (status_code, Json(error_response)).into_response()
    }
}

impl From<anyhow::Error> for AppError {
    fn from(err: anyhow::Error) -> Self {
        AppError::new(ErrorType::Internal(err.to_string()), err)
    }
}
pub trait IntoAppError<T> {
    fn into_db_error(self) -> Result<T, AppError>;
    fn into_auth_error(self) -> Result<T, AppError>;
    fn into_validation_error(self) -> Result<T, AppError>;
    fn into_not_found_error(self) -> Result<T, AppError>;
    fn into_internal_error(self) -> Result<T, AppError>;
}

impl<T, E> IntoAppError<T> for Result<T, E>
where
    E: Into<anyhow::Error>,
{
    fn into_db_error(self) -> Result<T, AppError> {
        self.map_err(|e| {
            let err: anyhow::Error = e.into();
            AppError::new(ErrorType::Database(err.to_string()), err)
        })
    }

    fn into_auth_error(self) -> Result<T, AppError> {
        self.map_err(|e| {
            let err: anyhow::Error = e.into();
            AppError::new(ErrorType::Authentication(err.to_string()), err)
        })
    }

    fn into_validation_error(self) -> Result<T, AppError> {
        self.map_err(|e| {
            let err: anyhow::Error = e.into();
            AppError::new(ErrorType::Validation(err.to_string()), err)
        })
    }

    fn into_not_found_error(self) -> Result<T, AppError> {
        self.map_err(|e| {
            let err: anyhow::Error = e.into();
            AppError::new(ErrorType::NotFound(err.to_string()), err)
        })
    }

    fn into_internal_error(self) -> Result<T, AppError> {
        self.map_err(|e| {
            let err: anyhow::Error = e.into();
            AppError::new(ErrorType::Internal(err.to_string()), err)
        })
    }
}

pub fn init_logging() {
    use tracing_appender::rolling::{RollingFileAppender, Rotation};
    use tracing_subscriber::{EnvFilter, layer::SubscriberExt, util::SubscriberInitExt};

    let env_filter = EnvFilter::try_from_default_env().unwrap_or_else(|_| {
        if cfg!(debug_assertions) {
            EnvFilter::new("debug")
        } else {
            EnvFilter::new("info")
        }
    });

    let log_dir = std::env::var("LOG_DIR").unwrap_or_else(|_| "./logs".to_string());

    let file_appender = RollingFileAppender::new(Rotation::DAILY, &log_dir, "spaces.log");

    tracing_subscriber::registry()
        .with(env_filter)
        .with(
            tracing_subscriber::fmt::layer()
                .json()
                .with_writer(file_appender),
        )
        .with(tracing_subscriber::fmt::layer().json())
        .init();
}
