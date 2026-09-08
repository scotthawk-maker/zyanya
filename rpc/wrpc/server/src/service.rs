use crate::{connection::*, router::*, server::*};
use async_trait::async_trait;
use std::sync::Arc;
use tokio::sync::oneshot::{channel as oneshot_channel, Sender as OneshotSender};
use workflow_rpc::server::prelude::*;
pub use workflow_rpc::server::{Encoding as WrpcEncoding, WebSocketConfig, WebSocketCounters};
use zyanya_core::{
    error, info,
    task::service::{AsyncService, AsyncServiceError, AsyncServiceFuture},
    trace, warn,
};
use zyanya_rpc_core::api::ops::RpcApiOps;
use zyanya_rpc_service::service::RpcCoreService;
use zyanya_utils::triggers::SingleTrigger;

pub const MAX_WRPC_MESSAGE_SIZE: usize = 64 * 1024 * 1024; // 64MB

/// Options for configuring the wRPC server
pub struct Options {
    pub listen_address: String,
    pub grpc_proxy_address: Option<String>,
    pub verbose: bool,
    /// F-C-13 FOLLOW-UP: optional bearer token for authenticating state-changing methods.
    /// When `Some`, wRPC clients must supply the token via the `Authorization: Bearer`
    /// header or `?token=` query parameter on the WebSocket upgrade request.
    pub rpc_auth_token: Option<String>,
}

impl Default for Options {
    fn default() -> Self {
        Options { listen_address: "127.0.0.1:19110".to_owned(), verbose: false, grpc_proxy_address: None, rpc_auth_token: None }
    }
}

/// ### ZyanyaRpcHandler
///
/// [`ZyanyaRpcHandler`] is a handler struct that implements the [`RpcHandler`] trait
/// allowing it to receive [`connect()`](RpcHandler::connect),
/// [`disconnect()`](RpcHandler::disconnect) and [`handshake()`](RpcHandler::handshake)
/// calls invoked by the [`RpcServer`].
///
/// [`RpcHandler::handshake`] is called by the [`RpcServer`] supplying the [`Messenger`]
/// and expecting user to return a `ServerContext` struct (or an `Arc` of) where
/// this struct will be supplied to each RPC method call.  Each RPC method call receives
/// 3 arguments - `ServerContext`, `ConnectionContext` and `Request`. Upon completion
/// the method should return a `Result`.
///
/// RPC method handling is implemented in the [`Router`].
///
pub struct ZyanyaRpcHandler {
    pub server: Server,
    pub options: Arc<Options>,
}

impl ZyanyaRpcHandler {
    pub fn new(
        tasks: usize,
        encoding: WrpcEncoding,
        core_service: Option<Arc<RpcCoreService>>,
        options: Arc<Options>,
    ) -> ZyanyaRpcHandler {
        ZyanyaRpcHandler { server: Server::new(tasks, encoding, core_service, options.clone()), options }
    }
}

#[async_trait]
impl RpcHandler for ZyanyaRpcHandler {
    type Context = Connection;

    async fn handshake(
        self: Arc<Self>,
        peer: &SocketAddr,
        _sender: &mut WebSocketSender,
        _receiver: &mut WebSocketReceiver,
        messenger: Arc<Messenger>,
    ) -> WebSocketResult<Connection> {
        // F-C-13 FOLLOW-UP: Extract the bearer token from the WebSocket HTTP upgrade
        // request (Authorization header or ?token= query parameter), captured by the
        // vendored workflow-websocket handshake callback.
        let auth_token = workflow_websocket::server::take_handshake_token(peer);

        let connection = self.server.connect(peer, messenger, auth_token).await.map_err(|err| err.to_string())?;
        Ok(connection)
    }

    /// Disconnect the websocket. Receives `Connection` (a.k.a `Self::Context`)
    /// before dropping it. This is the last chance to cleanup and resources owned by
    /// this connection. Delegate to Server.
    async fn disconnect(self: Arc<Self>, ctx: Self::Context, _result: WebSocketResult<()>) {
        // F-M-26: disconnect now returns Result; log any error instead of
        // ignoring a potential panic on a poisoned sockets mutex.
        if let Err(err) = self.server.disconnect(ctx).await {
            warn!("wRPC server disconnect error: {err}");
        }
    }
}

///
///  wRPC Server - A wrapper around and an initializer of the RpcServer
///
pub struct WrpcService {
    // TODO: see if tha Adapter/ConnectionHandler design of P2P and gRPC can be applied here too
    options: Arc<Options>,
    server: RpcServer,
    rpc_handler: Arc<ZyanyaRpcHandler>,
    shutdown: SingleTrigger,
}

impl WrpcService {
    /// Create and initialize RpcServer
    pub fn new(
        tasks: usize,
        core_service: Option<Arc<RpcCoreService>>,
        encoding: &Encoding,
        counters: Arc<WebSocketCounters>,
        options: Options,
    ) -> Self {
        let options = Arc::new(options);
        // Create handle to manage connections
        let rpc_handler = Arc::new(ZyanyaRpcHandler::new(tasks, *encoding, core_service, options.clone()));

        // Create router (initializes Interface registering RPC method and notification handlers)
        let router = Arc::new(Router::new(rpc_handler.server.clone()));
        // Create a server
        let server = RpcServer::new_with_encoding::<Server, Connection, RpcApiOps, Id64>(
            *encoding,
            rpc_handler.clone(),
            router.interface.clone(),
            Some(counters),
            false,
        );

        WrpcService { options, server, rpc_handler, shutdown: SingleTrigger::default() }
    }

    /// Start listening on the configured address (will panic if the socket listen() fails)
    pub fn serve(self: Arc<Self>) -> OneshotSender<()> {
        let (termination_sender, termination_receiver) = oneshot_channel::<()>();
        let listen_address = self.options.listen_address.clone();
        self.rpc_handler.server.start();

        // Spawn a task stopping the server on termination signal
        let service = self.clone();
        tokio::spawn(async move {
            let _ = termination_receiver.await;
            service.server.stop().unwrap_or_else(|err| warn!("wRPC unable to signal shutdown: `{err}`"));
            service.server.join().await.unwrap_or_else(|err| warn!("wRPC error: `{err}"));
        });

        // Spawn a task running the server
        info!("WRPC Server starting on: {}", listen_address);
        tokio::spawn(async move {
            let config = WebSocketConfig { max_message_size: Some(MAX_WRPC_MESSAGE_SIZE), ..Default::default() };
            match self.server.bind(&listen_address).await {
                Ok(listener) => {
                    let serve_result = self.server.listen(listener, Some(config)).await;
                    match serve_result {
                        Ok(_) => info!("WRPC Server stopped on: {}", listen_address),
                        // F-L-26: log instead of panicking on serve error.
                        Err(err) => error!("WRPC Server {listen_address} stopped with error: {err:?}"),
                    }
                }
                // F-L-26: log instead of panicking on bind error.
                Err(err) => error!("WRPC Server bind error on {listen_address}: {err:?}"),
            }
        });

        termination_sender
    }
}

const WRPC_SERVER: &str = "wrpc-service";

impl AsyncService for WrpcService {
    fn ident(self: Arc<Self>) -> &'static str {
        WRPC_SERVER
    }

    fn start(self: Arc<Self>) -> AsyncServiceFuture {
        trace!("{} starting", WRPC_SERVER);

        // Prepare a shutdown signal receiver
        let shutdown_signal = self.shutdown.listener.clone();

        // Run the server
        trace!("{} running the wRPC server", WRPC_SERVER);
        let terminate_server = self.clone().serve();

        Box::pin(async move {
            // Keep the gRPC server running until a service shutdown signal is received
            shutdown_signal.await;

            // Wait for the notifier to shutdown
            self.clone()
                .rpc_handler
                .server
                .join()
                .await
                .map_err(|err| AsyncServiceError::Service(format!("Notification system error: `{err}`")))?;

            // Signal server termination
            drop(terminate_server);

            Ok(())
        })
    }

    fn signal_exit(self: Arc<Self>) {
        trace!("sending an exit signal to {}", WRPC_SERVER);
        self.shutdown.trigger.trigger();
    }

    fn stop(self: Arc<Self>) -> AsyncServiceFuture {
        Box::pin(async move {
            trace!("{} stopped", WRPC_SERVER);
            Ok(())
        })
    }
}
