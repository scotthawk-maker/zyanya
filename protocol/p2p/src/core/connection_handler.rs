use crate::common::ProtocolError;
use crate::core::hub::HubEvent;
use crate::pb::{
    p2p_client::P2pClient as ProtoP2pClient, p2p_server::P2p as ProtoP2p, p2p_server::P2pServer as ProtoP2pServer, ZyanyadMessage,
};
use crate::{ConnectionInitializer, Router};
use futures::FutureExt;
use std::collections::{HashMap, VecDeque};
use std::net::ToSocketAddrs;
use std::pin::Pin;
use std::sync::{Arc, Mutex};
use std::task::{Context, Poll};
use std::time::{Duration, Instant};
use thiserror::Error;
use tokio::sync::mpsc::{channel as mpsc_channel, Sender as MpscSender};
use tokio::sync::oneshot::{channel as oneshot_channel, Sender as OneshotSender};
use tokio_stream::wrappers::ReceiverStream;
use tokio_stream::StreamExt;
use tonic::transport::{Error as TonicError, Server as TonicServer};
use tonic::{Request, Response, Status as TonicStatus, Streaming};
use zyanya_core::{debug, error, info, warn};
use zyanya_utils::networking::{IpAddress, NetAddress, PrefixBucket, PrefixBucket48};
use zyanya_utils_tower::{
    counters::TowerConnectionCounters,
    middleware::{BodyExt, CountBytesBody, MapRequestBodyLayer, MapResponseBodyLayer, ServiceBuilder},
};

/// Maximum number of concurrent inbound P2P connections (F-H-18).
const MAX_CONNECTIONS: usize = 128;
/// Maximum number of new inbound connections allowed per IP per minute (F-H-18).
const MAX_CONNECTIONS_PER_IP_PER_MINUTE: usize = 10;
/// Sliding window for the per-IP connection rate limit (F-H-18).
const CONNECTION_RATE_LIMIT_WINDOW: Duration = Duration::from_secs(60);

/// Maximum number of concurrent inbound connections allowed per /64 subnet (Track 10).
pub const MAX_INBOUND_PER_NETGROUP_64: usize = 1;
/// Maximum number of concurrent inbound connections allowed per /48 routing prefix (Track 10).
pub const MAX_INBOUND_PER_NETGROUP_48: usize = 4;
/// Maximum number of new inbound connection handshakes per /64 subnet per minute (Track 10).
pub const MAX_HANDSHAKES_PER_NETGROUP_64_PER_MINUTE: usize = 12;

/// Tracks active inbound connection counts and recent handshake rates partitioned by
/// IPv6 /64 and /48 prefix buckets to defend against subnet starvation and eclipse attacks (Track 10).
#[derive(Default, Debug)]
pub struct InboundPrefixLimiter {
    /// Active inbound connection count per /64 netgroup.
    pub active_64: HashMap<PrefixBucket, usize>,
    /// Active inbound connection count per /48 netgroup.
    pub active_48: HashMap<PrefixBucket48, usize>,
    /// Recent handshake timestamps per /64 netgroup for sliding-window rate limiting.
    pub handshakes_64: HashMap<PrefixBucket, VecDeque<Instant>>,
}

impl InboundPrefixLimiter {
    /// Evaluates whether an incoming connection from `ip` can be accepted.
    /// If accepted, increments active counters and records the handshake timestamp.
    pub fn try_accept(&mut self, ip: &IpAddress) -> Result<(), TonicStatus> {
        let now = Instant::now();
        let bucket_64 = ip.prefix_bucket();
        let bucket_48 = ip.prefix_bucket_48();

        // 1. Sliding window rate-limit check on /64 prefix (defends against rapid IP-cycling in a /64)
        let hs_entry = self.handshakes_64.entry(bucket_64).or_default();
        while hs_entry.front().map_or(false, |t| now.duration_since(*t) > CONNECTION_RATE_LIMIT_WINDOW) {
            hs_entry.pop_front();
        }
        if hs_entry.len() >= MAX_HANDSHAKES_PER_NETGROUP_64_PER_MINUTE {
            return Err(TonicStatus::resource_exhausted("IPv6 /64 subnet connection rate limit exceeded"));
        }

        // 2. Active concurrent connection limit for /64 subnet (max 1)
        let count_64 = self.active_64.get(&bucket_64).copied().unwrap_or(0);
        if count_64 >= MAX_INBOUND_PER_NETGROUP_64 {
            return Err(TonicStatus::resource_exhausted("IPv6 /64 subnet active inbound slot limit reached (max 1)"));
        }

        // 3. Active concurrent connection limit for /48 routing prefix (max 4)
        let count_48 = self.active_48.get(&bucket_48).copied().unwrap_or(0);
        if count_48 >= MAX_INBOUND_PER_NETGROUP_48 {
            return Err(TonicStatus::resource_exhausted("IPv6 /48 prefix active inbound slot limit reached (max 4)"));
        }

        // Accept and record
        hs_entry.push_back(now);
        *self.active_64.entry(bucket_64).or_insert(0) += 1;
        *self.active_48.entry(bucket_48).or_insert(0) += 1;

        Ok(())
    }

    /// Releases active connection slots when a peer disconnects.
    pub fn release(&mut self, ip: &IpAddress) {
        let bucket_64 = ip.prefix_bucket();
        let bucket_48 = ip.prefix_bucket_48();

        if let Some(count) = self.active_64.get_mut(&bucket_64) {
            if *count > 0 {
                *count -= 1;
                if *count == 0 {
                    self.active_64.remove(&bucket_64);
                }
            }
        }

        if let Some(count) = self.active_48.get_mut(&bucket_48) {
            if *count > 0 {
                *count -= 1;
                if *count == 0 {
                    self.active_48.remove(&bucket_48);
                }
            }
        }
    }
}

#[derive(Error, Debug)]
pub enum ConnectionError {
    #[error("missing socket address")]
    NoAddress,

    #[error("{0}")]
    IoError(#[from] std::io::Error),

    #[error("{0}")]
    TonicError(#[from] TonicError),

    #[error("{0}")]
    TonicStatus(#[from] TonicStatus),

    #[error("{0}")]
    ProtocolError(#[from] ProtocolError),
}

/// Maximum P2P decoded gRPC message size to send and receive (32 MB)
const P2P_MAX_MESSAGE_SIZE: usize = 32 * 1024 * 1024; // 32MB

/// Handles Router creation for both server and client-side new connections
#[derive(Clone)]
pub struct ConnectionHandler {
    /// Cloned on each new connection so that routers can communicate with a central hub
    hub_sender: MpscSender<HubEvent>,
    initializer: Arc<dyn ConnectionInitializer>,
    counters: Arc<TowerConnectionCounters>,
    /// Global inbound connection slots (released when a connection closes) (F-H-18).
    connection_slots: Arc<tokio::sync::Semaphore>,
    /// Per-IP timestamps of recent inbound connections (sliding window) (F-H-18).
    per_ip_connections: Arc<Mutex<HashMap<IpAddress, VecDeque<Instant>>>>,
    /// Inbound /64 and /48 prefix limiter and tracker (Track 10).
    prefix_limiter: Arc<Mutex<InboundPrefixLimiter>>,
}

impl ConnectionHandler {
    pub(crate) fn new(
        hub_sender: MpscSender<HubEvent>,
        initializer: Arc<dyn ConnectionInitializer>,
        counters: Arc<TowerConnectionCounters>,
    ) -> Self {
        Self {
            hub_sender,
            initializer,
            counters,
            connection_slots: Arc::new(tokio::sync::Semaphore::new(MAX_CONNECTIONS)),
            per_ip_connections: Arc::new(Mutex::new(HashMap::new())),
            prefix_limiter: Arc::new(Mutex::new(InboundPrefixLimiter::default())),
        }
    }

    /// Launches a P2P server listener loop
    pub(crate) fn serve(&self, serve_address: NetAddress) -> Result<OneshotSender<()>, ConnectionError> {
        let (termination_sender, termination_receiver) = oneshot_channel::<()>();
        let connection_handler = self.clone();
        info!("P2P Server starting on: {}", serve_address);

        let bytes_tx = self.counters.bytes_tx.clone();
        let bytes_rx = self.counters.bytes_rx.clone();

        tokio::spawn(async move {
            let proto_server = ProtoP2pServer::new(connection_handler)
                .accept_compressed(tonic::codec::CompressionEncoding::Gzip)
                .send_compressed(tonic::codec::CompressionEncoding::Gzip)
                .max_decoding_message_size(P2P_MAX_MESSAGE_SIZE)
                .max_encoding_message_size(P2P_MAX_MESSAGE_SIZE);

            // TODO: check whether we should set tcp_keepalive
            let serve_result = TonicServer::builder()
                .layer(MapRequestBodyLayer::new(move |body| CountBytesBody::new(body, bytes_rx.clone()).boxed_unsync()))
                .layer(MapResponseBodyLayer::new(move |body| CountBytesBody::new(body, bytes_tx.clone())))
                .add_service(proto_server)
                .serve_with_shutdown(serve_address.into(), termination_receiver.map(drop))
                .await;

            match serve_result {
                Ok(_) => info!("P2P Server stopped: {}", serve_address),
                // F-L-26: log instead of panicking on serve error.
                Err(err) => error!("P2P, Server {serve_address} stopped with error: {err:?}"),
            }
        });
        Ok(termination_sender)
    }

    /// Connect to a new peer
    pub(crate) async fn connect(&self, peer_address: String) -> Result<Arc<Router>, ConnectionError> {
        let Some(socket_address) = peer_address.to_socket_addrs()?.next() else {
            return Err(ConnectionError::NoAddress);
        };

        match socket_address.ip() {
            std::net::IpAddr::V4(_) => {
                return Err(ConnectionError::ProtocolError(ProtocolError::Other("Pure IPv6 invariant violation: IPv4 connections are rejected")));
            }
            std::net::IpAddr::V6(v6) => {
                if v6.to_ipv4_mapped().is_some() || (v6.to_ipv4().is_some() && !v6.is_loopback() && !v6.is_unspecified()) {
                    return Err(ConnectionError::ProtocolError(ProtocolError::Other("Pure IPv6 invariant violation: IPv4-mapped IPv6 connections are rejected")));
                }
            }
        }

        let peer_address = format!("http://{}", peer_address); // Add scheme prefix as required by Tonic

        let channel = tonic::transport::Endpoint::new(peer_address)?
            .timeout(Duration::from_millis(Self::communication_timeout()))
            .connect_timeout(Duration::from_millis(Self::connect_timeout()))
            .tcp_keepalive(Some(Duration::from_millis(Self::keep_alive())))
            .connect()
            .await?;

        let channel = ServiceBuilder::new()
            .layer(MapResponseBodyLayer::new(move |body| CountBytesBody::new(body, self.counters.bytes_rx.clone())))
            .layer(MapRequestBodyLayer::new(move |body| CountBytesBody::new(body, self.counters.bytes_tx.clone()).boxed_unsync()))
            .service(channel);

        let mut client = ProtoP2pClient::new(channel)
            .send_compressed(tonic::codec::CompressionEncoding::Gzip)
            .accept_compressed(tonic::codec::CompressionEncoding::Gzip)
            .max_decoding_message_size(P2P_MAX_MESSAGE_SIZE)
            .max_encoding_message_size(P2P_MAX_MESSAGE_SIZE);

        let (outgoing_route, outgoing_receiver) = mpsc_channel(Self::outgoing_network_channel_size());
        let incoming_stream = client.message_stream(ReceiverStream::new(outgoing_receiver)).await?.into_inner();

        let router = Router::new(socket_address, true, self.hub_sender.clone(), incoming_stream, outgoing_route).await;

        // For outbound peers, we perform the initialization as part of the connect logic
        match self.initializer.initialize_connection(router.clone()).await {
            Ok(()) => {
                // Notify the central Hub about the new peer
                // F-L-25: log and continue instead of panicking if the hub receiver dropped.
                if self.hub_sender.send(HubEvent::NewPeer(router.clone())).await.is_err() {
                    warn!("hub receiver dropped; peer notification skipped");
                }
            }

            Err(err) => {
                router.try_sending_reject_message(&err).await;
                // Ignoring the new router
                router.close().await;
                debug!("P2P, handshake failed for outbound peer {}: {}", router, err);
                return Err(ConnectionError::ProtocolError(err));
            }
        }

        Ok(router)
    }

    /// Connect to a new peer with `retry_attempts` retries and `retry_interval` duration between each attempt
    pub(crate) async fn connect_with_retry(
        &self,
        address: String,
        retry_attempts: u8,
        retry_interval: Duration,
    ) -> Result<Arc<Router>, ConnectionError> {
        let mut counter = 0;
        loop {
            counter += 1;
            match self.connect(address.clone()).await {
                Ok(router) => {
                    debug!("P2P, Client connected, peer: {:?}", address);
                    return Ok(router);
                }
                Err(ConnectionError::ProtocolError(err)) => {
                    // On protocol errors we avoid retrying
                    debug!("P2P, connect retry #{} failed with error {:?}, peer: {:?}, aborting retries", counter, err, address);
                    return Err(ConnectionError::ProtocolError(err));
                }
                Err(err) => {
                    debug!("P2P, connect retry #{} failed with error {:?}, peer: {:?}", counter, err, address);
                    if counter < retry_attempts {
                        // Await `retry_interval` time before retrying
                        tokio::time::sleep(retry_interval).await;
                    } else {
                        debug!("P2P, Client connection retry #{} - all failed", retry_attempts);
                        return Err(err);
                    }
                }
            }
        }
    }

    // TODO: revisit the below constants
    fn outgoing_network_channel_size() -> usize {
        // TODO: this number is taken from go-zyanyad and should be re-evaluated
        (1 << 17) + 256
    }

    fn communication_timeout() -> u64 {
        10_000
    }

    fn keep_alive() -> u64 {
        10_000
    }

    fn connect_timeout() -> u64 {
        1_000
    }
}

/// RAII guard that decrements active /64 and /48 prefix connection counters upon drop (Track 10).
pub struct PrefixSlotGuard {
    ip: IpAddress,
    limiter: Arc<Mutex<InboundPrefixLimiter>>,
}

impl Drop for PrefixSlotGuard {
    fn drop(&mut self) {
        let mut limiter = self.limiter.lock().unwrap();
        limiter.release(&self.ip);
    }
}

/// Wraps the outgoing stream and holds connection permits so both global slots and
/// subnet/routing prefix slots are released when the connection drops (Track 10).
struct ConnectionGuardStream<S> {
    inner: S,
    _permit: tokio::sync::OwnedSemaphorePermit,
    _prefix_guard: PrefixSlotGuard,
}

impl<S: futures::Stream + Unpin> futures::Stream for ConnectionGuardStream<S> {
    type Item = S::Item;
    fn poll_next(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        Pin::new(&mut self.get_mut().inner).poll_next(cx)
    }
}

#[tonic::async_trait]
impl ProtoP2p for ConnectionHandler {
    type MessageStreamStream = Pin<Box<dyn futures::Stream<Item = Result<ZyanyadMessage, TonicStatus>> + Send + 'static>>;

    /// Handle the new arriving **server** connections
    async fn message_stream(
        &self,
        request: Request<Streaming<ZyanyadMessage>>,
    ) -> Result<Response<Self::MessageStreamStream>, TonicStatus> {
        let Some(remote_address) = request.remote_addr() else {
            return Err(TonicStatus::new(tonic::Code::InvalidArgument, "Incoming connection opening request has no remote address"));
        };

        match remote_address.ip() {
            std::net::IpAddr::V4(_) => {
                return Err(TonicStatus::invalid_argument("Pure IPv6 invariant violation: IPv4 connections are rejected"));
            }
            std::net::IpAddr::V6(v6) => {
                if v6.to_ipv4_mapped().is_some() || (v6.to_ipv4().is_some() && !v6.is_loopback() && !v6.is_unspecified()) {
                    return Err(TonicStatus::invalid_argument("Pure IPv6 invariant violation: IPv4-mapped IPv6 connections are rejected"));
                }
            }
        }

        let ip: IpAddress = remote_address.ip().into();

        // 1. Inbound /64 and /48 prefix limit and rate limit check (Track 10).
        {
            let mut limiter = self.prefix_limiter.lock().unwrap();
            limiter.try_accept(&ip)?;
        }
        let prefix_guard = PrefixSlotGuard {
            ip,
            limiter: self.prefix_limiter.clone(),
        };

        // 2. Per-IP rate limit (sliding window) (F-H-18).
        {
            let mut map = self.per_ip_connections.lock().unwrap();
            let now = Instant::now();
            let entry = map.entry(ip).or_default();
            while entry.front().map_or(false, |t| now.duration_since(*t) > CONNECTION_RATE_LIMIT_WINDOW) {
                entry.pop_front();
            }
            if entry.len() >= MAX_CONNECTIONS_PER_IP_PER_MINUTE {
                return Err(TonicStatus::resource_exhausted("per-IP connection rate limit exceeded"));
            }
            entry.push_back(now);
        }

        // 3. Global inbound connection limit (F-H-18).
        let permit = match self.connection_slots.clone().try_acquire_owned() {
            Ok(permit) => permit,
            Err(_) => return Err(TonicStatus::resource_exhausted("max connections reached")),
        };

        // Build the in/out pipes
        let (outgoing_route, outgoing_receiver) = mpsc_channel(Self::outgoing_network_channel_size());
        let incoming_stream = request.into_inner();

        // Build the router object
        let router = Router::new(remote_address, false, self.hub_sender.clone(), incoming_stream, outgoing_route).await;

        // Notify the central Hub about the new peer
        // F-L-25: log and continue instead of panicking if the hub receiver dropped.
        if self.hub_sender.send(HubEvent::NewPeer(router)).await.is_err() {
            warn!("hub receiver dropped; peer notification skipped");
        }

        // Give tonic a receiver stream (messages sent to it will be forwarded to the network peer).
        // Wrap the stream with a ConnectionGuardStream so both global and prefix slots are
        // automatically released when the connection drops.
        let stream = ReceiverStream::new(outgoing_receiver).map(Ok);
        let guarded = ConnectionGuardStream { inner: stream, _permit: permit, _prefix_guard: prefix_guard };
        Ok(Response::new(Box::pin(guarded) as Self::MessageStreamStream))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::net::Ipv6Addr;

    #[test]
    fn test_prefix_limiter_enforces_per_64_limit() {
        let mut limiter = InboundPrefixLimiter::default();

        let ip1 = IpAddress::from(std::net::IpAddr::V6("2606:4700:4700::1".parse::<Ipv6Addr>().unwrap()));
        let ip2 = IpAddress::from(std::net::IpAddr::V6("2606:4700:4700::2".parse::<Ipv6Addr>().unwrap()));

        // First connection from /64: accepted
        assert!(limiter.try_accept(&ip1).is_ok());

        // Second connection from same /64: rejected
        let res = limiter.try_accept(&ip2);
        assert!(res.is_err());
        assert_eq!(res.unwrap_err().code(), tonic::Code::ResourceExhausted);

        // Release first connection
        limiter.release(&ip1);

        // Now ip2 from that /64 can be accepted
        assert!(limiter.try_accept(&ip2).is_ok());
    }

    #[test]
    fn test_prefix_limiter_enforces_per_48_limit() {
        let mut limiter = InboundPrefixLimiter::default();

        // 5 distinct /64 subnets all within 2606:4700:4700::/48
        let ips = (0..5)
            .map(|i| {
                IpAddress::from(std::net::IpAddr::V6(
                    format!("2606:4700:4700:{:04x}::1", i).parse::<Ipv6Addr>().unwrap(),
                ))
            })
            .collect::<Vec<_>>();

        // First 4 (max 4 per /48): all accepted
        for ip in &ips[0..4] {
            assert!(limiter.try_accept(ip).is_ok());
        }

        // 5th connection from same /48: rejected even though /64 is new
        let res5 = limiter.try_accept(&ips[4]);
        assert!(res5.is_err());
        assert_eq!(res5.unwrap_err().code(), tonic::Code::ResourceExhausted);

        // Release one slot
        limiter.release(&ips[0]);

        // 5th connection now succeeds
        assert!(limiter.try_accept(&ips[4]).is_ok());
    }

    #[test]
    fn test_pure_ipv6_invariant_validation() {
        let v4: std::net::IpAddr = "192.168.1.1".parse().unwrap();
        let v4_mapped: std::net::IpAddr = "::ffff:192.168.1.1".parse().unwrap();
        let v6_valid: std::net::IpAddr = "2606:4700:4700::1".parse().unwrap();
        let v6_loopback: std::net::IpAddr = "::1".parse().unwrap();

        let is_valid_ipv6 = |addr: std::net::IpAddr| -> bool {
            match addr {
                std::net::IpAddr::V4(_) => false,
                std::net::IpAddr::V6(v6) => {
                    !(v6.to_ipv4_mapped().is_some() || (v6.to_ipv4().is_some() && !v6.is_loopback() && !v6.is_unspecified()))
                }
            }
        };

        assert!(!is_valid_ipv6(v4));
        assert!(!is_valid_ipv6(v4_mapped));
        assert!(is_valid_ipv6(v6_valid));
        assert!(is_valid_ipv6(v6_loopback));
    }
}
