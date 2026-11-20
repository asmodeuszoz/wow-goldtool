//! One error type for the whole app: the CLI prints it, commands serialise it,
//! the sync records it in `sync_state`. Keeping it small means the UI can show the
//! message without re-deriving error shapes.

use serde::Serialize;

#[derive(Debug, Clone, Serialize)]
pub struct AppError {
    pub kind: ErrorKind,
    pub message: String,
    pub status: Option<u16>,
    pub retry_after: Option<u32>,
}

#[derive(Debug, Clone, Serialize)]
pub enum ErrorKind {
    Usage,
    Network,
    RateLimited,
    Db,
    Decode,
    NotFound,
}

impl AppError {
    pub fn make(kind: ErrorKind, message: String) -> AppError {
        AppError {
            kind,
            message,
            status: None,
            retry_after: None,
        }
    }

    pub fn usage(message: String) -> AppError {
        AppError::make(ErrorKind::Usage, message)
    }

    pub fn network(message: String) -> AppError {
        AppError::make(ErrorKind::Network, message)
    }

    pub fn db(message: String) -> AppError {
        AppError::make(ErrorKind::Db, message)
    }

    pub fn decode(message: String) -> AppError {
        AppError::make(ErrorKind::Decode, message)
    }

    pub fn not_found(message: String) -> AppError {
        AppError::make(ErrorKind::NotFound, message)
    }

    pub fn rate_limited(message: String, retry_after: Option<u32>) -> AppError {
        AppError {
            kind: ErrorKind::RateLimited,
            message,
            status: None,
            retry_after,
        }
    }

    pub fn http(status: u16, message: String) -> AppError {
        AppError {
            kind: ErrorKind::Network,
            message,
            status: Some(status),
            retry_after: None,
        }
    }

    /// Human-readable form for the CLI's stderr line and the UI's error banner.
    pub fn to_text(&self) -> String {
        match self.kind {
            ErrorKind::RateLimited => format!("rate limited by the API: {}", self.message),
            _ => self.message.clone(),
        }
    }
}

/// Every fallible step in the data layer returns this; the CLI maps the kinds onto
/// exit codes (docs/DECISIONS/0004) and the commands map them onto JSON error objects.
pub type AppResult<T> = Result<T, AppError>;
