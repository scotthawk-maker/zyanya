use zyanya_notify::error::Error as NotifyError;
use zyanya_rpc_core::RpcError;
use std::sync::PoisonError;
use thiserror::Error;
use workflow_rpc::server::{error::Error as RpcServerError, WebSocketError};

#[derive(Debug, Error)]
pub enum Error {
    #[error("RpcServer error: {0}")]
    RpcServerError(#[from] RpcServerError),

    #[error("WebSocket error: {0}")]
    WebSocketError(#[from] WebSocketError),

    #[error("Poison error")]
    PoisonError,

    #[error("RPC error: {0}")]
    RpcError(RpcError),

    #[error("Notify error: {0}")]
    NotifyError(#[from] NotifyError),
}

// F-M-28: sanitize RpcError before it reaches unauthenticated wRPC clients.
// The full error should already have been logged server-side by the handler.
impl From<RpcError> for Error {
    fn from(err: RpcError) -> Self {
        Error::RpcError(err.sanitize())
    }
}

impl<T> From<PoisonError<T>> for Error {
    fn from(_: PoisonError<T>) -> Self {
        Error::PoisonError
    }
}
