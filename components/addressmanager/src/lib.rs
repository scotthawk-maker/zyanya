mod port_mapping_extender;
mod stores;
extern crate self as address_manager;

use std::{collections::HashSet, iter, net::SocketAddr, sync::Arc, time::Duration};

use address_manager::port_mapping_extender::Extender;
use igd_next::{
    self as igd, aio::tokio::Tokio, AddAnyPortError, AddPortError, Gateway, GetExternalIpError, GetGenericPortMappingEntryError,
    SearchError,
};
use itertools::{
    Either::{Left, Right},
    Itertools,
};
use local_ip_address::list_afinet_netifas;
use parking_lot::Mutex;
use stores::banned_address_store::{BannedAddressesStore, BannedAddressesStoreReader, ConnectionBanTimestamp, DbBannedAddressesStore};
use thiserror::Error;
use zyanya_consensus_core::config::Config;
use zyanya_core::{debug, info, task::tick::TickService, time::unix_now, warn};
use zyanya_database::prelude::{CachePolicy, StoreResultExtensions, DB};
use zyanya_utils::networking::IpAddress;

pub use stores::NetAddress;

const MAX_ADDRESSES: usize = 4096;
const MAX_CONNECTION_FAILED_COUNT: u64 = 3;

/// F-M-25: helper used by [`RandomWeightedIterator`] to log a non-fatal
/// `WeightedError` instead of panicking.
fn log_weighted_error(e: rand::distributions::WeightedError) {
    warn!("RandomWeightedIterator weight update failed: {e}; degrading to empty iterator");
}

/// Initial failure count for addresses learned from trusted sources
/// (connected peers, DNS seeds, local addresses).
const TRUSTED_INITIAL_FAILED_COUNT: u64 = 1;
/// Initial failure count for addresses learned from untrusted sources
/// (peer-advertised AddressesMessage). Equal to MAX_CONNECTION_FAILED_COUNT so
/// untrusted addresses are evicted first and dropped after one failure.
const UNTRUSTED_INITIAL_FAILED_COUNT: u64 = MAX_CONNECTION_FAILED_COUNT;

const UPNP_DEADLINE_SEC: u64 = 2 * 60;
const UPNP_EXTEND_PERIOD: u64 = UPNP_DEADLINE_SEC / 2;

/// The name used as description when registering the UPnP service
pub(crate) const UPNP_REGISTRATION_NAME: &str = "rusty-zyanya";

struct ExtendHelper {
    gateway: Gateway,
    local_addr: SocketAddr,
    external_port: u16,
}

#[derive(Error, Debug)]
pub enum UpnpError {
    #[error(transparent)]
    AddPortError(#[from] AddPortError),
    #[error(transparent)]
    AddAnyPortError(#[from] AddAnyPortError),
    #[error(transparent)]
    SearchError(#[from] SearchError),
    #[error(transparent)]
    GetExternalIpError(#[from] GetExternalIpError),
}

pub struct AddressManager {
    banned_address_store: DbBannedAddressesStore,
    address_store: address_store_with_cache::Store,
    config: Arc<Config>,
    local_net_addresses: Vec<NetAddress>,
}

impl AddressManager {
    pub fn new(config: Arc<Config>, db: Arc<DB>, tick_service: Arc<TickService>) -> (Arc<Mutex<Self>>, Option<Extender>) {
        let mut instance = Self {
            banned_address_store: DbBannedAddressesStore::new(db.clone(), CachePolicy::Count(MAX_ADDRESSES)),
            address_store: address_store_with_cache::new(db),
            local_net_addresses: Vec::new(),
            config,
        };

        let extender = instance.init_local_addresses(tick_service);

        (Arc::new(Mutex::new(instance)), extender)
    }

    fn init_local_addresses(&mut self, tick_service: Arc<TickService>) -> Option<Extender> {
        self.local_net_addresses = self.local_addresses().collect();

        let extender = if self.local_net_addresses.is_empty() && !self.config.disable_upnp {
            let (net_address, ExtendHelper { gateway, local_addr, external_port }) = match self.upnp() {
                Err(err) => {
                    warn!("[UPnP] Error adding port mapping: {err}");
                    return None;
                }
                Ok(None) => return None,
                Ok(Some((net_address, extend_helper))) => (net_address, extend_helper),
            };
            self.local_net_addresses.push(net_address);

            let gateway: igd_next::aio::Gateway<Tokio> = igd_next::aio::Gateway {
                addr: gateway.addr,
                root_url: gateway.root_url,
                control_url: gateway.control_url,
                control_schema_url: gateway.control_schema_url,
                control_schema: gateway.control_schema,
                provider: Tokio,
            };
            Some(Extender::new(
                tick_service,
                Duration::from_secs(UPNP_EXTEND_PERIOD),
                UPNP_DEADLINE_SEC,
                gateway,
                external_port,
                local_addr,
            ))
        } else {
            None
        };

        self.local_net_addresses.iter().for_each(|net_addr| {
            info!("Publicly routable local address {} added to store", net_addr);
        });
        extender
    }

    fn local_addresses(&self) -> impl Iterator<Item = NetAddress> + '_ {
        match self.config.externalip {
            // An external IP was passed, we will try to bind that if it's valid
            Some(local_net_address) if local_net_address.ip.is_publicly_routable() => {
                info!("External address is publicly routable {}", local_net_address);
                return Left(iter::once(local_net_address));
            }
            Some(local_net_address) => {
                info!("External address is not publicly routable {}", local_net_address);
            }
            None => {}
        };

        Right(self.routable_addresses_from_net_interfaces())
    }

    fn routable_addresses_from_net_interfaces(&self) -> impl Iterator<Item = NetAddress> + '_ {
        // check whatever was passed as listen address (if routable)
        // otherwise(listen_address === 0.0.0.0) check all interfaces
        let listen_address = self.config.p2p_listen_address.normalize(self.config.default_p2p_port());
        if listen_address.ip.is_publicly_routable() {
            info!("Publicly routable local address found: {}", listen_address.ip);
            Left(Left(iter::once(listen_address)))
        } else if listen_address.ip.is_unspecified() {
            let network_interfaces = list_afinet_netifas();
            let Ok(network_interfaces) = network_interfaces else {
                warn!("Error getting network interfaces: {:?}", network_interfaces);
                return Left(Right(iter::empty()));
            };
            // TODO: Add Check IPv4 or IPv6 match from Go code
            Right(network_interfaces.into_iter().map(|(_, ip)| IpAddress::from(ip)).filter(|&ip| ip.is_publicly_routable()).map(
                |ip| {
                    info!("Publicly routable local address found: {}", ip);
                    NetAddress::new(ip, self.config.default_p2p_port())
                },
            ))
        } else {
            Left(Right(iter::empty()))
        }
    }

    fn upnp(&self) -> Result<Option<(NetAddress, ExtendHelper)>, UpnpError> {
        info!("[UPnP] Attempting to register upnp... (to disable run the node with --disable-upnp)");
        let gateway = igd::search_gateway(Default::default())?;
        let ip = IpAddress::new(gateway.get_external_ip()?);
        if !ip.is_publicly_routable() {
            info!("[UPnP] Non-publicly routable external ip from gateway using upnp {} not added to store", ip);
            return Ok(None);
        }
        info!("[UPnP] Got external ip from gateway using upnp: {ip}");

        let normalized_p2p_listen_address = self.config.p2p_listen_address.normalize(self.config.default_p2p_port());
        let local_addr = if normalized_p2p_listen_address.ip.is_unspecified() {
            SocketAddr::new(local_ip_address::local_ip().unwrap(), normalized_p2p_listen_address.port)
        } else {
            normalized_p2p_listen_address.into()
        };

        // If an operator runs a node and specifies a non-standard local port, it implies that they also wish to use a non-standard public address. The variable 'desired_external_port' is set to the port number from the normalized peer-to-peer listening address.
        let desired_external_port = normalized_p2p_listen_address.port;
        // This loop checks for existing port mappings in the UPnP-enabled gateway.
        //
        // The goal of this loop is to identify if the desired external port (`desired_external_port`) is
        // already mapped to any device inside the local network. This is crucial because, in
        // certain scenarios, gateways might not throw the `PortInUse` error but rather might
        // silently remap the external port when there's a conflict. By iterating through the
        // current mappings, we can make an informed decision about whether to attempt using
        // the default port or request a new random one.
        //
        // The loop goes through all existing port mappings one-by-one:
        // - If a mapping is found that uses the desired external port, the loop breaks with `already_in_use` set to true.
        // - If the index is not valid (i.e., we've iterated through all the mappings), the loop breaks with `already_in_use` set to false.
        // - Any other errors during fetching of port mappings are handled accordingly, but the end result is to exit the loop with the `already_in_use` flag set appropriately.
        let mut index = 0;
        let already_in_use = loop {
            match gateway.get_generic_port_mapping_entry(index) {
                Ok(entry) => {
                    if entry.enabled && entry.external_port == desired_external_port {
                        info!("[UPnP] Found existing mapping that uses the same external port. Description: {}, external port: {}, internal port: {}, client: {}, lease duration: {}", entry.port_mapping_description, entry.external_port, entry.internal_port, entry.internal_client, entry.lease_duration);
                        break true;
                    }
                    index += 1;
                }
                Err(GetGenericPortMappingEntryError::ActionNotAuthorized) => {
                    index += 1;
                    continue;
                }
                Err(GetGenericPortMappingEntryError::RequestError(err)) => {
                    warn!("[UPnP] request existing port mapping err: {:?}", err);
                    break false;
                }
                Err(GetGenericPortMappingEntryError::SpecifiedArrayIndexInvalid) => break false,
            }
        };
        if already_in_use {
            let port =
                gateway.add_any_port(igd::PortMappingProtocol::TCP, local_addr, UPNP_DEADLINE_SEC as u32, UPNP_REGISTRATION_NAME)?;
            info!("[UPnP] Added port mapping to random external port: {ip}:{port}");
            return Ok(Some((NetAddress { ip, port }, ExtendHelper { gateway, local_addr, external_port: port })));
        }

        match gateway.add_port(
            igd::PortMappingProtocol::TCP,
            desired_external_port,
            local_addr,
            UPNP_DEADLINE_SEC as u32,
            UPNP_REGISTRATION_NAME,
        ) {
            Ok(_) => {
                info!("[UPnP] Added port mapping to default external port: {ip}:{desired_external_port}");
                Ok(Some((
                    NetAddress { ip, port: desired_external_port },
                    ExtendHelper { gateway, local_addr, external_port: desired_external_port },
                )))
            }
            Err(AddPortError::PortInUse) => {
                let port = gateway.add_any_port(
                    igd::PortMappingProtocol::TCP,
                    local_addr,
                    UPNP_DEADLINE_SEC as u32,
                    UPNP_REGISTRATION_NAME,
                )?;
                info!("[UPnP] Added port mapping to random external port: {ip}:{port}");
                Ok(Some((NetAddress { ip, port }, ExtendHelper { gateway, local_addr, external_port: port })))
            }
            Err(err) => Err(err.into()),
        }
    }

    pub fn best_local_address(&mut self) -> Option<NetAddress> {
        if self.local_net_addresses.is_empty() {
            None
        } else {
            // TODO: Add logic for finding the best as a function of a peer remote address.
            // for now, returning the first one
            Some(self.local_net_addresses[0])
        }
    }

    pub fn add_address(&mut self, address: NetAddress, trusted: bool) {
        if address.ip.is_loopback() || address.ip.is_unspecified() {
            debug!("[Address manager] skipping local address {}", address.ip);
            return;
        }
        // Reject invalid ports (port 0 is never a valid peer address).
        if address.port == 0 {
            debug!("[Address manager] skipping address with invalid port {}", address);
            return;
        }

        if self.address_store.has(address) {
            return;
        }

        // We mark `connection_failed_count` as 0 only after first success.
        // Untrusted addresses start with a high failure count so they are evicted
        // first and dropped after a single connection failure.
        let initial_count = if trusted { TRUSTED_INITIAL_FAILED_COUNT } else { UNTRUSTED_INITIAL_FAILED_COUNT };
        self.address_store.set(address, initial_count);
    }

    /// Updates the set of currently-connected peer addresses, so that eviction
    /// (`keep_limit`) never displaces a connected peer.
    pub fn set_connected(&mut self, connected: HashSet<NetAddress>) {
        self.address_store.set_connected(connected);
    }

    pub fn mark_connection_failure(&mut self, address: NetAddress) {
        if !self.address_store.has(address) {
            return;
        }

        let new_count = self.address_store.get(address).connection_failed_count.saturating_add(1);
        if new_count > MAX_CONNECTION_FAILED_COUNT {
            self.address_store.remove(address);
        } else {
            self.address_store.set(address, new_count);
        }
    }

    pub fn mark_connection_success(&mut self, address: NetAddress) {
        if !self.address_store.has(address) {
            return;
        }

        self.address_store.set(address, 0);
    }

    pub fn iterate_addresses(&self) -> impl Iterator<Item = NetAddress> + '_ {
        self.address_store.iterate_addresses()
    }

    pub fn iterate_prioritized_random_addresses(
        &self,
        exceptions: HashSet<NetAddress>,
    ) -> Result<impl ExactSizeIterator<Item = NetAddress>, rand::distributions::WeightedError> {
        self.address_store.iterate_prioritized_random_addresses(exceptions)
    }

    pub fn ban(&mut self, ip: IpAddress) {
        self.banned_address_store.set(ip.into(), ConnectionBanTimestamp(unix_now())).unwrap();
        self.address_store.remove_by_ip(ip.into());
    }

    pub fn unban(&mut self, ip: IpAddress) {
        self.banned_address_store.remove(ip.into()).unwrap();
    }

    pub fn is_banned(&mut self, ip: IpAddress) -> bool {
        const MAX_BANNED_TIME: u64 = 24 * 60 * 60 * 1000;
        match self.banned_address_store.get(ip.into()).unwrap_option() {
            Some(timestamp) => {
                // Use saturating_sub to avoid u64 underflow when the ban timestamp
                // is in the future (clock skew or manipulation). A future timestamp
                // yields elapsed == 0, keeping the peer banned until the clock
                // catches up — i.e. no ban bypass.
                let elapsed = unix_now().saturating_sub(timestamp.0);
                if elapsed > MAX_BANNED_TIME {
                    self.unban(ip);
                    false
                } else {
                    true
                }
            }
            None => false,
        }
    }

    pub fn get_all_addresses(&self) -> Vec<NetAddress> {
        self.address_store.iterate_addresses().collect_vec()
    }

    pub fn get_all_banned_addresses(&self) -> Vec<IpAddress> {
        self.banned_address_store.iterator().map(|x| IpAddress::from(x.unwrap().0)).collect_vec()
    }
}

mod address_store_with_cache {
    // Since we need operations such as iterating all addresses, count, etc, we keep an easy to use copy of the database addresses.
    // We don't expect it to be expensive since we limit the number of saved addresses.
    use std::{
        collections::{HashMap, HashSet},
        net::IpAddr,
        sync::Arc,
    };

    use itertools::Itertools;
    use rand::{
        distributions::{WeightedError, WeightedIndex},
        prelude::Distribution,
    };
    use zyanya_database::prelude::{CachePolicy, DB};
    use zyanya_utils::networking::PrefixBucket;

    use crate::{
        stores::{
            address_store::{AddressesStore, DbAddressesStore, Entry},
            AddressKey,
        },
        NetAddress, MAX_ADDRESSES, MAX_CONNECTION_FAILED_COUNT,
    };

    pub struct Store {
        db_store: DbAddressesStore,
        addresses: HashMap<AddressKey, Entry>,
        /// Set of addresses that are currently connected, used by `keep_limit`
        /// to protect connected peers from eviction.
        connected: HashSet<AddressKey>,
    }

    impl Store {
        fn new(db: Arc<DB>) -> Self {
            // We manage the cache ourselves on this level, so we disable the inner builtin cache
            let db_store = DbAddressesStore::new(db, CachePolicy::Empty);
            let mut addresses = HashMap::new();
            for (key, entry) in db_store.iterator().map(|res| res.unwrap()) {
                addresses.insert(key, entry);
            }

            Self { db_store, addresses, connected: HashSet::new() }
        }

        pub fn set_connected(&mut self, connected: HashSet<NetAddress>) {
            self.connected = connected.into_iter().map(|a| a.into()).collect();
        }

        pub fn has(&mut self, address: NetAddress) -> bool {
            self.addresses.contains_key(&address.into())
        }

        pub fn set(&mut self, address: NetAddress, connection_failed_count: u64) {
            let entry = match self.addresses.get(&address.into()) {
                Some(entry) => Entry { connection_failed_count, address: entry.address },
                None => Entry { connection_failed_count, address },
            };
            self.db_store.set(address.into(), entry).unwrap();
            self.addresses.insert(address.into(), entry);
            self.keep_limit();
        }

        fn keep_limit(&mut self) {
            while self.addresses.len() > MAX_ADDRESSES {
                // Evict the highest-failure-count address that is NOT currently
                // connected, so connected peers are protected from eviction.
                // Only fall back to connected peers if every address is connected.
                let to_remove = self
                    .addresses
                    .iter()
                    .filter(|(key, _)| !self.connected.contains(key))
                    .max_by(|a, b| (a.1).connection_failed_count.cmp(&(b.1).connection_failed_count))
                    .or_else(|| self.addresses.iter().max_by(|a, b| (a.1).connection_failed_count.cmp(&(b.1).connection_failed_count)))
                    .map(|(key, _)| *key)
                    .unwrap();
                self.remove_by_key(to_remove);
            }
        }

        pub fn get(&self, address: NetAddress) -> Entry {
            *self.addresses.get(&address.into()).unwrap()
        }

        pub fn remove(&mut self, address: NetAddress) {
            self.remove_by_key(address.into())
        }

        fn remove_by_key(&mut self, key: AddressKey) {
            self.addresses.remove(&key);
            self.connected.remove(&key);
            self.db_store.remove(key).unwrap()
        }

        pub fn iterate_addresses(&self) -> impl Iterator<Item = NetAddress> + '_ {
            self.addresses.values().map(|entry| entry.address)
        }

        /// This iterator functions as the node's ip routing selection algo.
        /// It first adjusts in respect to the number of connection failures of each ip address,
        /// whereby each connection failure (up to [`MAX_CONNECTION_FAILED_COUNT`]) reduces an ip's selection weight by a factor of 64,
        /// Afterwards the weights are normalized uniformly over the ip's [`PrefixBucket`] size.
        ///
        /// This ensures a distributed selection across the global network, while respecting
        /// weight reductions due to ip connection failures.
        ///
        /// The exact weight formula for any given ip, is as follows:
        ///```ignore
        ///         ip_weight = (64 ^ (x - y)) / n
        ///
        ///             whereby:
        ///                 x: max allowed connection failures.
        ///                 y: connection failures of the ip.
        ///                 n: number of ips with the same prefix bytes.
        ///```
        pub fn iterate_prioritized_random_addresses(
            &self,
            exceptions: HashSet<NetAddress>,
        ) -> Result<impl ExactSizeIterator<Item = NetAddress>, rand::distributions::WeightedError> {
            let exceptions: HashSet<AddressKey> = exceptions.into_iter().map(|addr| addr.into()).collect();
            let mut prefix_counter: HashMap<PrefixBucket, usize> = HashMap::new();
            let (mut weights, filtered_addresses): (Vec<f64>, Vec<NetAddress>) = self
                .addresses
                .iter()
                .filter(|(addr_key, _)| !exceptions.contains(addr_key))
                .map(|(_, e)| {
                    let count = prefix_counter.entry(e.address.prefix_bucket()).or_insert(0);
                    *count += 1;
                    (64f64.powf((MAX_CONNECTION_FAILED_COUNT + 1 - e.connection_failed_count) as f64), e.address)
                })
                .unzip();

            // Divide weights by size of bucket of the prefix bytes, to partially uniform the distribution over prefix buckets.
            for (i, address) in filtered_addresses.iter().enumerate() {
                *weights.get_mut(i).unwrap() /= *prefix_counter.get(&address.prefix_bucket()).unwrap() as f64;
            }

            RandomWeightedIterator::new(weights, filtered_addresses)
        }

        pub fn remove_by_ip(&mut self, ip: IpAddr) {
            for key in self.addresses.keys().filter(|key| key.is_ip(ip)).copied().collect_vec() {
                self.remove_by_key(key);
            }
        }
    }

    pub fn new(db: Arc<DB>) -> Store {
        Store::new(db)
    }

    pub struct RandomWeightedIterator {
        weighted_index: Option<WeightedIndex<f64>>,
        remaining: usize,
        addresses: Vec<NetAddress>,
    }

    impl RandomWeightedIterator {
        /// F-M-25: returns `Err` for a non-recoverable `WeightedError` instead of
        /// panicking, so callers (connection manager) can degrade gracefully.
        pub fn new(weights: Vec<f64>, addresses: Vec<NetAddress>) -> Result<Self, WeightedError> {
            assert_eq!(weights.len(), addresses.len());
            let remaining = weights.iter().filter(|&&w| w > 0.0).count();
            let weighted_index = match WeightedIndex::new(weights) {
                Ok(index) => Some(index),
                Err(WeightedError::NoItem) => None,
                Err(e) => return Err(e),
            };
            Ok(Self { weighted_index, remaining, addresses })
        }
    }

    impl Iterator for RandomWeightedIterator {
        type Item = NetAddress;

        fn next(&mut self) -> Option<Self::Item> {
            if let Some(weighted_index) = self.weighted_index.as_mut() {
                let i = weighted_index.sample(&mut rand::thread_rng());
                // Zero the selected address entry
                match weighted_index.update_weights(&[(i, &0f64)]) {
                    Ok(_) => {}
                    Err(WeightedError::AllWeightsZero) => self.weighted_index = None,
                    // F-M-25: log and degrade to an empty iterator instead of
                    // panicking on an invalid weight update.
                    Err(e) => {
                        crate::log_weighted_error(e);
                        self.weighted_index = None;
                    }
                }
                self.remaining -= 1;
                if self.remaining == 0 {
                    self.weighted_index = None;
                }
                Some(self.addresses[i])
            } else {
                None
            }
        }

        fn size_hint(&self) -> (usize, Option<usize>) {
            (self.remaining, Some(self.remaining))
        }
    }

    impl ExactSizeIterator for RandomWeightedIterator {}

    #[cfg(test)]
    mod tests {
        use std::str::FromStr;

        use super::*;
        use crate::stores::banned_address_store::{BannedAddressesStore, BannedAddressesStoreReader, ConnectionBanTimestamp};
        use address_manager::AddressManager;
        use rv::{dist::Uniform, misc::ks_test as one_way_ks_test, traits::Cdf};
        use std::net::{IpAddr, Ipv6Addr};
        use zyanya_consensus_core::config::{params::SIMNET_PARAMS, Config};
        use zyanya_core::{task::tick::TickService, time::unix_now};
        use zyanya_database::create_temp_db;
        use zyanya_database::prelude::ConnBuilder;
        use zyanya_database::prelude::StoreResultExtensions;
        use zyanya_utils::networking::IpAddress;

        #[test]
        fn test_weighted_iterator() {
            let address = NetAddress::new(IpAddr::V6(Ipv6Addr::LOCALHOST).into(), 1);
            let iter = RandomWeightedIterator::new(vec![0.2, 0.3, 0.0], vec![address, address, address]).unwrap();
            assert_eq!(iter.len(), 2);
            assert_eq!(iter.count(), 2);

            let iter = RandomWeightedIterator::new(vec![], vec![]).unwrap();
            assert_eq!(iter.len(), 0);
            assert_eq!(iter.count(), 0);
        }

        #[test]
        fn test_network_distribution_weighting() {
            zyanya_core::log::try_init_logger("info");

            // Variables to initialize ip generation with.
            let largest_bucket: u16 = 2048;
            let bucket_reduction_ratio: f64 = 2.;

            // Assert that initial distribution is skewed, and hence not uniform from the outset.
            assert!(bucket_reduction_ratio >= 1.25);

            let db = create_temp_db!(ConnBuilder::default().with_files_limit(10));
            let config = Config::new(SIMNET_PARAMS);
            let (am, _) = AddressManager::new(Arc::new(config), db.1, Arc::new(TickService::default()));

            let mut am_guard = am.lock();

            let mut num_of_buckets = 0;
            let mut num_of_addresses = 0;
            let mut current_bucket_size = largest_bucket;

            for current_prefix_bytes in 0..u16::MAX {
                num_of_buckets += 1;
                for current_suffix_bytes in 0..current_bucket_size {
                    let current_ip_bytes =
                        [current_prefix_bytes.to_be_bytes(), current_suffix_bytes.to_be_bytes()].concat().to_owned();
                    am_guard.add_address(
                        NetAddress::new(
                            IpAddress::from_str(&format!(
                                "{0}.{1}.{2}.{3}",
                                current_ip_bytes[0], current_ip_bytes[1], current_ip_bytes[2], current_ip_bytes[3]
                            ))
                            .unwrap(),
                            18111,
                        ),
                        true,
                    );
                    num_of_addresses += 1;
                }

                let last_bucket_size = current_bucket_size;
                current_bucket_size = ((current_bucket_size as f64) * (1.0 / bucket_reduction_ratio)).round() as u16;

                if current_bucket_size == last_bucket_size || current_bucket_size == 0 || current_prefix_bytes == u16::MAX {
                    // Address generation exhausted - exit loop
                    break;
                }
            }
            drop(am_guard);

            // Assert sample size is large enough.
            assert!(1024 <= num_of_addresses);
            // Assert we don't over-generate the address manager's limit.
            assert!(num_of_addresses <= MAX_ADDRESSES);
            // Assert that the test has enough buckets to sample from
            assert!(num_of_buckets >= 12);

            // Run multiple Kolmogorov–Smirnov tests to offset random noise of the random weighted iterator
            let num_of_trials = 2048; // Number of trials to run the test, chosen to reduce random noise.
            let mut cul_p = 0.;
            // The target uniform distribution
            let target_uniform_dist = Uniform::new(1.0, num_of_buckets as f64).unwrap();
            let uniform_cdf = |x: f64| target_uniform_dist.cdf(&x);
            for _ in 0..num_of_trials {
                // The weight sampled expected uniform distribution
                let prioritized_address_distribution = am
                    .lock()
                    .iterate_prioritized_random_addresses(HashSet::new())
                    .unwrap()
                    .take(num_of_buckets)
                    .map(|addr| addr.prefix_bucket().as_u64() as f64)
                    .collect_vec();
                cul_p += one_way_ks_test(prioritized_address_distribution.as_slice(), uniform_cdf).1;
            }

            // Normalize and adjust p to test for uniformity, over average of all trials.
            // we do this to reduce the effect of random noise failing this test.
            let adjusted_p = ((cul_p / num_of_trials as f64) - 0.5).abs();
            // Define the significance threshold.
            let significance = 0.10;

            // Display and assert the result
            zyanya_core::info!(
                "Kolmogorov–Smirnov test result for weighted network distribution uniformity: p = {0:.4} (p < {1})",
                adjusted_p,
                significance
            );
            assert!(adjusted_p <= significance);
        }

        #[test]
        fn test_is_banned_future_timestamp_no_underflow() {
            // F-H-22: a ban with a future timestamp must NOT cause a u64 underflow
            // that bypasses the ban.
            let db = create_temp_db!(ConnBuilder::default().with_files_limit(10));
            let config = Config::new(SIMNET_PARAMS);
            let (am, _) = AddressManager::new(Arc::new(config), db.1, Arc::new(TickService::default()));
            let mut am_guard = am.lock();

            let ip = IpAddress::from_str("1.2.3.4").unwrap();

            // Insert a ban with a timestamp 10 seconds in the future.
            let future_ts = unix_now() + 10_000;
            am_guard.banned_address_store.set(ip.into(), ConnectionBanTimestamp(future_ts)).unwrap();

            // The peer should remain banned (no underflow bypass).
            assert!(am_guard.is_banned(ip));
        }

        #[test]
        fn test_is_banned_expired_is_unbanned() {
            let db = create_temp_db!(ConnBuilder::default().with_files_limit(10));
            let config = Config::new(SIMNET_PARAMS);
            let (am, _) = AddressManager::new(Arc::new(config), db.1, Arc::new(TickService::default()));
            let mut am_guard = am.lock();

            let ip = IpAddress::from_str("5.6.7.8").unwrap();

            // Insert a ban with a timestamp 25 hours ago (beyond the 24h MAX_BANNED_TIME).
            const MAX_BANNED_TIME: u64 = 24 * 60 * 60 * 1000;
            let old_ts = unix_now().saturating_sub(MAX_BANNED_TIME + 60_000);
            am_guard.banned_address_store.set(ip.into(), ConnectionBanTimestamp(old_ts)).unwrap();

            // The ban should be expired and removed.
            assert!(!am_guard.is_banned(ip));
            assert!(am_guard.banned_address_store.get(ip.into()).unwrap_option().is_none());
        }

        #[test]
        fn test_add_address_rejects_invalid_port() {
            let db = create_temp_db!(ConnBuilder::default().with_files_limit(10));
            let config = Config::new(SIMNET_PARAMS);
            let (am, _) = AddressManager::new(Arc::new(config), db.1, Arc::new(TickService::default()));
            let mut am_guard = am.lock();

            let ip = IpAddress::from_str("8.8.8.8").unwrap();
            // Port 0 should be rejected.
            am_guard.add_address(NetAddress::new(ip, 0), true);
            assert_eq!(am_guard.get_all_addresses().len(), 0);

            // A valid port should be accepted.
            am_guard.add_address(NetAddress::new(ip, 18111), true);
            assert_eq!(am_guard.get_all_addresses().len(), 1);
        }

        #[test]
        fn test_untrusted_addresses_evicted_first() {
            // F-H-21: untrusted addresses (high failure count) should be evicted
            // before trusted addresses (low failure count).
            let db = create_temp_db!(ConnBuilder::default().with_files_limit(10));
            let config = Config::new(SIMNET_PARAMS);
            let (am, _) = AddressManager::new(Arc::new(config), db.1, Arc::new(TickService::default()));
            let mut am_guard = am.lock();

            // Add a trusted address (failure count = 1).
            let trusted_ip = IpAddress::from_str("10.0.0.1").unwrap();
            am_guard.add_address(NetAddress::new(trusted_ip, 18111), true);

            // Add an untrusted address (failure count = MAX_CONNECTION_FAILED_COUNT).
            let untrusted_ip = IpAddress::from_str("10.0.0.2").unwrap();
            am_guard.add_address(NetAddress::new(untrusted_ip, 18111), false);

            // Both should be present.
            assert_eq!(am_guard.get_all_addresses().len(), 2);

            // Fill the store up to MAX_ADDRESSES with trusted addresses so the next
            // add triggers eviction. The untrusted address (highest failure count)
            // should be evicted first.
            // We need MAX_ADDRESSES - 1 more addresses (already have 2) to exceed
            // the limit and trigger eviction. Use subnet 11.x to avoid collisions
            // with the 10.x test addresses.
            for i in 0..(MAX_ADDRESSES - 1) as u32 {
                let high = (i / 256) as u8;
                let low = (i % 256) as u8;
                let ip = IpAddress::from_str(&format!("11.{}.{}.1", high, low)).unwrap();
                am_guard.add_address(NetAddress::new(ip, 18111), true);
            }

            // The store should be at MAX_ADDRESSES.
            assert_eq!(am_guard.get_all_addresses().len(), MAX_ADDRESSES);

            // The untrusted address should have been evicted (it had the highest
            // connection_failed_count).
            let remaining: Vec<NetAddress> = am_guard.get_all_addresses();
            assert!(remaining.iter().all(|a| a.ip != untrusted_ip), "untrusted address should have been evicted first");
            // The trusted address should still be present.
            assert!(remaining.iter().any(|a| a.ip == trusted_ip), "trusted address should survive eviction");
        }

        #[test]
        fn test_connected_peer_not_evicted() {
            // F-H-21: a connected peer must never be evicted by keep_limit.
            let db = create_temp_db!(ConnBuilder::default().with_files_limit(10));
            let config = Config::new(SIMNET_PARAMS);
            let (am, _) = AddressManager::new(Arc::new(config), db.1, Arc::new(TickService::default()));
            let mut am_guard = am.lock();

            // Add a high-failure-count (untrusted) address and mark it as connected.
            let connected_ip = IpAddress::from_str("192.168.1.1").unwrap();
            let connected_addr = NetAddress::new(connected_ip, 18111);
            am_guard.add_address(connected_addr, false);

            let mut connected_set = HashSet::new();
            connected_set.insert(connected_addr);
            am_guard.set_connected(connected_set);

            // Fill the store past MAX_ADDRESSES with trusted (low-failure) addresses.
            // Use subnet 11.x to avoid collision with the 192.168 connected address.
            for i in 0..MAX_ADDRESSES as u32 {
                let high = (i / 256) as u8;
                let low = (i % 256) as u8;
                let ip = IpAddress::from_str(&format!("11.{}.{}.1", high, low)).unwrap();
                am_guard.add_address(NetAddress::new(ip, 18111), true);
            }

            assert_eq!(am_guard.get_all_addresses().len(), MAX_ADDRESSES);

            // The connected address must still be present even though it had the
            // highest failure count.
            let remaining: Vec<NetAddress> = am_guard.get_all_addresses();
            assert!(remaining.iter().any(|a| a.ip == connected_ip), "connected peer must not be evicted");
        }
    }
}
