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
