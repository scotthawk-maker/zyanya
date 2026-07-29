use super::error::Result;
use core::fmt::Debug;
use zyanya_grpc_core::{
    ops::ZyanyadPayloadOps,
    protowire::{ZyanyadRequest, ZyanyadResponse},
};
use std::{sync::Arc, time::Duration};
use tokio::sync::oneshot;

pub(crate) mod id;
pub(crate) mod matcher;
pub(crate) mod queue;

pub(crate) trait Resolver: Send + Sync + Debug {
    fn register_request(&self, op: ZyanyadPayloadOps, request: &ZyanyadRequest) -> ZyanyadResponseReceiver;
    fn handle_response(&self, response: ZyanyadResponse);
    fn remove_expired_requests(&self, timeout: Duration);
}

pub(crate) type DynResolver = Arc<dyn Resolver>;

pub(crate) type ZyanyadResponseSender = oneshot::Sender<Result<ZyanyadResponse>>;
pub(crate) type ZyanyadResponseReceiver = oneshot::Receiver<Result<ZyanyadResponse>>;
