use std::sync::Arc;
pub use stoat_result::{Error as StoatHttpError, ErrorType as StoatHttpErrorType};

use crate::types::RatelimitFailure;

pub type Result<T, E = Error> = std::result::Result<T, E>;

#[derive(Debug, Clone)]
pub enum Error {
    ReqwestError(Arc<reqwest::Error>),
    WsError(Arc<tungstenite::Error>),
    HttpError(StoatHttpError),
    RatelimitReached(RatelimitFailure),
    InternalError,
    ClosedWs,
    ClosedWsLocal,
}

impl From<reqwest::Error> for Error {
    fn from(value: reqwest::Error) -> Self {
        Self::ReqwestError(Arc::new(value))
    }
}

impl From<tungstenite::Error> for Error {
    fn from(value: tungstenite::Error) -> Self {
        Self::WsError(Arc::new(value))
    }
}

impl PartialEq for Error {
    fn eq(&self, other: &Self) -> bool {
        core::mem::discriminant(self) == core::mem::discriminant(other)
    }
}

pub fn format_error<'a>(error: &Error, context: impl Into<Option<&'a str>>) -> String {
    let context = context.into();

    match error {
        Error::ReqwestError(error) => format!("Request Failed: {:?}", error.as_ref()),
        Error::WsError(error) => format!("Internal Error: {:?}", error.as_ref()),
        Error::HttpError(error) => {
            use stoat_result::ErrorType::*;
            match &error.error_type {
                InvalidCredentials => "Provided email or password is wrong.".to_string(),
                NotFound => {
                    if let Some(context) = context {
                        format!("Unknown {context}.")
                    } else {
                        "Not found.".to_string()
                    }
                }
                error => format!("{error:?}"),
            }
        }
        Error::RatelimitReached(ratelimit_failure) => format!(
            "Ratelimit reached, try again in {:.2}s.",
            ratelimit_failure.retry_after as f32 / 1000.
        ),
        Error::InternalError | Error::ClosedWs | Error::ClosedWsLocal => {
            "Internal Client Error".to_string()
        }
    }
}
