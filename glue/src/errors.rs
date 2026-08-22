use serde::{Deserialize, Serialize};
use std::fmt;
use thiserror::Error;

#[cfg(feature = "actix")] // compiles only with the actix feature
use actix_web::{HttpResponse, error::ResponseError, http::StatusCode};

#[derive(Error, Debug, Serialize, Deserialize, PartialEq)]
pub enum NanoServiceErrorStatus {
    // error derive macro implements the Display trait for our enum
    #[error("Requested resource was not found")]
    NotFound,
    #[error("You are forbidden to access requested resource.")]
    Forbidden,
    #[error("Unknown Internal Error")]
    Unknown,
    #[error("Bad Request")]
    BadRequest,
    #[error("Conflict")]
    Conflict,
    #[error("Unauthorized")]
    Unauthorized,
}

#[derive(Serialize, Deserialize, Debug, Error)]
pub struct NanoServiceError {
    pub message: String,
    pub status: NanoServiceErrorStatus,
}

impl NanoServiceError {
    pub fn new(message: String, status: NanoServiceErrorStatus) -> NanoServiceError {
        NanoServiceError { message, status }
    }
}

impl fmt::Display for NanoServiceError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.message)
    }
}

// Implementing the ResponseError trait for our NanoServiceError struct enables
// the NanoServiceError to be converted into an HTTP response with a message and error code
// so we can return the NanoServiceError back to the client
#[cfg(feature = "actix")]
impl ResponseError for NanoServiceError {
    fn status_code(&self) -> StatusCode {
        match self.status {
            NanoServiceErrorStatus::NotFound => StatusCode::NOT_FOUND,
            NanoServiceErrorStatus::Forbidden => StatusCode::FORBIDDEN,
            NanoServiceErrorStatus::Unknown => StatusCode::INTERNAL_SERVER_ERROR,
            NanoServiceErrorStatus::BadRequest => StatusCode::BAD_REQUEST,
            NanoServiceErrorStatus::Conflict => StatusCode::CONFLICT,
            NanoServiceErrorStatus::Unauthorized => StatusCode::UNAUTHORIZED,
        }
    }
    fn error_response(&self) -> HttpResponse {
        let status_code = self.status_code();
        HttpResponse::build(status_code).json(self.message.clone())
    }
}

#[macro_export]
macro_rules! safe_eject {
    // If we just pass in the expression to be mapped and the error status, the first branch will be triggered
    ($e:expr, $err_status:expr) => {
        $e.map_err(|x| NanoServiceError::new(x.to_string(), $err_status))
    };
    // if we pass in the expression to be mapped, the status, and a message, the second arm will be triggered
    ($e:expr, $err_status:expr, $message_context:expr) => {
        $e.map_err(|x| {
            NanoServiceError::new(
                format!("{}: {}", $message_context, x.to_string()),
                $err_status,
            )
        })
    };
}
