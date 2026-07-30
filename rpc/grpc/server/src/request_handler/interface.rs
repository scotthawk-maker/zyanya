use super::method::{DropFn, Method, MethodTrait, RoutingPolicy};
use crate::{
    connection::Connection,
    connection_handler::ServerContext,
    error::{GrpcServerError, GrpcServerResult},
};
use zyanya_grpc_core::{
    ops::ZyanyadPayloadOps,
    protowire::{ZyanyadRequest, ZyanyadResponse},
};
use std::fmt::Debug;
use std::{collections::HashMap, sync::Arc};

pub type ZyanyadMethod = Method<ServerContext, Connection, ZyanyadRequest, ZyanyadResponse>;
pub type DynZyanyadMethod = Arc<dyn MethodTrait<ServerContext, Connection, ZyanyadRequest, ZyanyadResponse>>;
pub type ZyanyadDropFn = DropFn<ZyanyadRequest, ZyanyadResponse>;
pub type ZyanyadRoutingPolicy = RoutingPolicy<ZyanyadRequest, ZyanyadResponse>;

/// An interface providing methods implementations and a fallback "not implemented" method
/// actually returning a message with a "not implemented" error.
///
/// The interface can provide a method clone for every [`ZyanyadPayloadOps`] variant for later
/// processing of related requests.
///
/// It is also possible to directly let the interface itself process a request by invoking
/// the `call()` method.
pub struct Interface {
    server_ctx: ServerContext,
    methods: HashMap<ZyanyadPayloadOps, DynZyanyadMethod>,
    method_not_implemented: DynZyanyadMethod,
}

impl Interface {
    pub fn new(server_ctx: ServerContext) -> Self {
        let method_not_implemented = Arc::new(Method::new(|_, _, zyanyad_request: ZyanyadRequest| {
            Box::pin(async move {
                match zyanyad_request.payload {
                    Some(ref request) => Ok(ZyanyadResponse {
                        id: zyanyad_request.id,
                        payload: Some(
                            ZyanyadPayloadOps::from(request).to_error_response(GrpcServerError::MethodNotImplemented.into()),
                        ),
                    }),
                    None => Err(GrpcServerError::InvalidRequestPayload),
                }
            })
        }));
        Self { server_ctx, methods: Default::default(), method_not_implemented }
    }

    pub fn method(&mut self, op: ZyanyadPayloadOps, method: ZyanyadMethod) {
        let method: DynZyanyadMethod = Arc::new(method);
        if self.methods.insert(op, method).is_some() {
            panic!("RPC method {op:?} is declared multiple times")
        }
    }

    pub fn replace_method(&mut self, op: ZyanyadPayloadOps, method: ZyanyadMethod) {
        let method: DynZyanyadMethod = Arc::new(method);
        let _ = self.methods.insert(op, method);
    }

    pub fn set_method_properties(
        &mut self,
        op: ZyanyadPayloadOps,
        tasks: usize,
        queue_size: usize,
        routing_policy: ZyanyadRoutingPolicy,
    ) {
        self.methods.entry(op).and_modify(|x| {
            let method: Method<ServerContext, Connection, ZyanyadRequest, ZyanyadResponse> =
                Method::with_properties(x.method_fn(), tasks, queue_size, routing_policy);
            let method: Arc<dyn MethodTrait<ServerContext, Connection, ZyanyadRequest, ZyanyadResponse>> = Arc::new(method);
            *x = method;
        });
    }

    pub async fn call(
        &self,
        op: &ZyanyadPayloadOps,
        connection: Connection,
        request: ZyanyadRequest,
    ) -> GrpcServerResult<ZyanyadResponse> {
        self.methods.get(op).unwrap_or(&self.method_not_implemented).call(self.server_ctx.clone(), connection, request).await
    }

    pub fn get_method(&self, op: &ZyanyadPayloadOps) -> DynZyanyadMethod {
        self.methods.get(op).unwrap_or(&self.method_not_implemented).clone()
    }
}

impl Debug for Interface {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Interface").finish()
    }
}
