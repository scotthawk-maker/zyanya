pub mod channel;
pub mod convert;
pub mod ext;
pub mod macros;
pub mod ops;

/// Maximum decoded gRPC message size to send and receive (64 MB)
pub const RPC_MAX_MESSAGE_SIZE: usize = 64 * 1024 * 1024;

pub mod protowire {
    tonic::include_proto!("protowire");
}
