use zyanya_grpc_core::ops::ZyanyadPayloadOps;
use thiserror::Error;
use tokio::sync::mpsc::error::TrySendError;

#[derive(Debug, Error)]
pub enum GrpcServerError {
    #[error("RpcApi error: {0}")]
    RpcApiError(#[from] zyanya_rpc_core::error::RpcError),

    #[error("Notification subsystem error: {0}")]
    NotificationError(#[from] zyanya_notify::error::Error),

    #[error("Request has no valid payload")]
    InvalidRequestPayload,

    #[error("Subscription has no valid payload")]
    InvalidSubscriptionPayload,

    #[error("This RPC method is not implemented by the gRPC server")]
    MethodNotImplemented,

    #[error("{0:?} handler is closed")]
    ClosedHandler(ZyanyadPayloadOps),

    #[error("client connection is closed")]
    ConnectionClosed,

    #[error("outgoing route capacity has been reached (client: {0})")]
    OutgoingRouteCapacityReached(String),
}

impl From<GrpcServerError> for zyanya_rpc_core::error::RpcError {
    fn from(err: GrpcServerError) -> Self {
        match err {
            GrpcServerError::RpcApiError(err) => err,
            GrpcServerError::NotificationError(err) => err.into(),
            _ => zyanya_rpc_core::error::RpcError::General(err.to_string()),
        }
    }
}

impl From<GrpcServerError> for zyanya_notify::error::Error {
    fn from(err: GrpcServerError) -> Self {
        match err {
            GrpcServerError::RpcApiError(err) => zyanya_notify::error::Error::General(err.to_string()),
            GrpcServerError::NotificationError(err) => err,
            _ => zyanya_notify::error::Error::General(err.to_string()),
        }
    }
}

impl<T> From<TrySendError<T>> for GrpcServerError {
    fn from(_: TrySendError<T>) -> Self {
        zyanya_notify::error::Error::ChannelSendError.into()
    }
}

pub type GrpcServerResult<T> = std::result::Result<T, GrpcServerError>;
