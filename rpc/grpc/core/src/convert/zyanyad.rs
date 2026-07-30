use crate::protowire::{zyanyad_request, ZyanyadRequest, ZyanyadResponse};

impl From<zyanyad_request::Payload> for ZyanyadRequest {
    fn from(item: zyanyad_request::Payload) -> Self {
        ZyanyadRequest { id: 0, payload: Some(item) }
    }
}

impl AsRef<ZyanyadRequest> for ZyanyadRequest {
    fn as_ref(&self) -> &Self {
        self
    }
}

impl AsRef<ZyanyadResponse> for ZyanyadResponse {
    fn as_ref(&self) -> &Self {
        self
    }
}

pub mod zyanyad_request_convert {
    use crate::protowire::*;
    use zyanya_rpc_core::{RpcError, RpcResult};

    impl_into_zyanyad_request!(Shutdown);
    impl_into_zyanyad_request!(SubmitBlock);
    impl_into_zyanyad_request!(GetBlockTemplate);
    impl_into_zyanyad_request!(GetBlock);
    impl_into_zyanyad_request!(GetInfo);

    impl_into_zyanyad_request!(GetCurrentNetwork);
    impl_into_zyanyad_request!(GetPeerAddresses);
    impl_into_zyanyad_request!(GetSink);
    impl_into_zyanyad_request!(GetMempoolEntry);
    impl_into_zyanyad_request!(GetMempoolEntries);
    impl_into_zyanyad_request!(GetConnectedPeerInfo);
    impl_into_zyanyad_request!(AddPeer);
    impl_into_zyanyad_request!(SubmitTransaction);
    impl_into_zyanyad_request!(SubmitTransactionReplacement);
    impl_into_zyanyad_request!(GetSubnetwork);
    impl_into_zyanyad_request!(GetVirtualChainFromBlock);
    impl_into_zyanyad_request!(GetBlocks);
    impl_into_zyanyad_request!(GetBlockCount);
    impl_into_zyanyad_request!(GetBlockDagInfo);
    impl_into_zyanyad_request!(ResolveFinalityConflict);
    impl_into_zyanyad_request!(GetHeaders);
    impl_into_zyanyad_request!(GetUtxosByAddresses);
    impl_into_zyanyad_request!(GetBalanceByAddress);
    impl_into_zyanyad_request!(GetBalancesByAddresses);
    impl_into_zyanyad_request!(GetSinkBlueScore);
    impl_into_zyanyad_request!(Ban);
    impl_into_zyanyad_request!(Unban);
    impl_into_zyanyad_request!(EstimateNetworkHashesPerSecond);
    impl_into_zyanyad_request!(GetMempoolEntriesByAddresses);
    impl_into_zyanyad_request!(GetCoinSupply);
    impl_into_zyanyad_request!(Ping);
    impl_into_zyanyad_request!(GetMetrics);
    impl_into_zyanyad_request!(GetConnections);
    impl_into_zyanyad_request!(GetSystemInfo);
    impl_into_zyanyad_request!(GetServerInfo);
    impl_into_zyanyad_request!(GetSyncStatus);
    impl_into_zyanyad_request!(GetDaaScoreTimestampEstimate);
    impl_into_zyanyad_request!(GetFeeEstimate);
    impl_into_zyanyad_request!(GetFeeEstimateExperimental);
    impl_into_zyanyad_request!(GetCurrentBlockColor);
    impl_into_zyanyad_request!(GetUtxoReturnAddress);
    impl_into_zyanyad_request!(DeployContract);
    impl_into_zyanyad_request!(InvokeContract);
    impl_into_zyanyad_request!(GetContractState);
    impl_into_zyanyad_request!(GetContractCode);
    impl_into_zyanyad_request!(CallContract);

    impl_into_zyanyad_request!(NotifyBlockAdded);
    impl_into_zyanyad_request!(NotifyNewBlockTemplate);
    impl_into_zyanyad_request!(NotifyUtxosChanged);
    impl_into_zyanyad_request!(NotifyPruningPointUtxoSetOverride);
    impl_into_zyanyad_request!(NotifyFinalityConflict);
    impl_into_zyanyad_request!(NotifyVirtualDaaScoreChanged);
    impl_into_zyanyad_request!(NotifyVirtualChainChanged);
    impl_into_zyanyad_request!(NotifySinkBlueScoreChanged);

    macro_rules! impl_into_zyanyad_request {
        ($name:tt) => {
            paste::paste! {
                impl_into_zyanyad_request_ex!(zyanya_rpc_core::[<$name Request>],[<$name RequestMessage>],[<$name Request>]);
            }
        };
    }

    use impl_into_zyanyad_request;

    macro_rules! impl_into_zyanyad_request_ex {
        // ($($core_struct:ident)::+, $($protowire_struct:ident)::+, $($variant:ident)::+) => {
        ($core_struct:path, $protowire_struct:ident, $variant:ident) => {
            // ----------------------------------------------------------------------------
            // rpc_core to protowire
            // ----------------------------------------------------------------------------

            impl From<&$core_struct> for zyanyad_request::Payload {
                fn from(item: &$core_struct) -> Self {
                    Self::$variant(item.into())
                }
            }

            impl From<&$core_struct> for ZyanyadRequest {
                fn from(item: &$core_struct) -> Self {
                    Self { id: 0, payload: Some(item.into()) }
                }
            }

            impl From<$core_struct> for zyanyad_request::Payload {
                fn from(item: $core_struct) -> Self {
                    Self::$variant((&item).into())
                }
            }

            impl From<$core_struct> for ZyanyadRequest {
                fn from(item: $core_struct) -> Self {
                    Self { id: 0, payload: Some((&item).into()) }
                }
            }

            // ----------------------------------------------------------------------------
            // protowire to rpc_core
            // ----------------------------------------------------------------------------

            impl TryFrom<&zyanyad_request::Payload> for $core_struct {
                type Error = RpcError;
                fn try_from(item: &zyanyad_request::Payload) -> RpcResult<Self> {
                    if let zyanyad_request::Payload::$variant(request) = item {
                        request.try_into()
                    } else {
                        Err(RpcError::MissingRpcFieldError("Payload".to_string(), stringify!($variant).to_string()))
                    }
                }
            }

            impl TryFrom<&ZyanyadRequest> for $core_struct {
                type Error = RpcError;
                fn try_from(item: &ZyanyadRequest) -> RpcResult<Self> {
                    item.payload
                        .as_ref()
                        .ok_or(RpcError::MissingRpcFieldError("ZyanyaRequest".to_string(), "Payload".to_string()))?
                        .try_into()
                }
            }

            impl From<$protowire_struct> for ZyanyadRequest {
                fn from(item: $protowire_struct) -> Self {
                    Self { id: 0, payload: Some(zyanyad_request::Payload::$variant(item)) }
                }
            }

            impl From<$protowire_struct> for zyanyad_request::Payload {
                fn from(item: $protowire_struct) -> Self {
                    zyanyad_request::Payload::$variant(item)
                }
            }
        };
    }
    use impl_into_zyanyad_request_ex;
}

pub mod zyanyad_response_convert {
    use crate::protowire::*;
    use zyanya_rpc_core::{RpcError, RpcResult};

    impl_into_zyanyad_response!(Shutdown);
    impl_into_zyanyad_response!(SubmitBlock);
    impl_into_zyanyad_response!(GetBlockTemplate);
    impl_into_zyanyad_response!(GetBlock);
    impl_into_zyanyad_response!(GetInfo);
    impl_into_zyanyad_response!(GetCurrentNetwork);

    impl_into_zyanyad_response!(GetPeerAddresses);
    impl_into_zyanyad_response!(GetSink);
    impl_into_zyanyad_response!(GetMempoolEntry);
    impl_into_zyanyad_response!(GetMempoolEntries);
    impl_into_zyanyad_response!(GetConnectedPeerInfo);
    impl_into_zyanyad_response!(AddPeer);
    impl_into_zyanyad_response!(SubmitTransaction);
    impl_into_zyanyad_response!(SubmitTransactionReplacement);
    impl_into_zyanyad_response!(GetSubnetwork);
    impl_into_zyanyad_response!(GetVirtualChainFromBlock);
    impl_into_zyanyad_response!(GetBlocks);
    impl_into_zyanyad_response!(GetBlockCount);
    impl_into_zyanyad_response!(GetBlockDagInfo);
    impl_into_zyanyad_response!(ResolveFinalityConflict);
    impl_into_zyanyad_response!(GetHeaders);
    impl_into_zyanyad_response!(GetUtxosByAddresses);
    impl_into_zyanyad_response!(GetBalanceByAddress);
    impl_into_zyanyad_response!(GetBalancesByAddresses);
    impl_into_zyanyad_response!(GetSinkBlueScore);
    impl_into_zyanyad_response!(Ban);
    impl_into_zyanyad_response!(Unban);
    impl_into_zyanyad_response!(EstimateNetworkHashesPerSecond);
    impl_into_zyanyad_response!(GetMempoolEntriesByAddresses);
    impl_into_zyanyad_response!(GetCoinSupply);
    impl_into_zyanyad_response!(Ping);
    impl_into_zyanyad_response!(GetMetrics);
    impl_into_zyanyad_response!(GetConnections);
    impl_into_zyanyad_response!(GetSystemInfo);
    impl_into_zyanyad_response!(GetServerInfo);
    impl_into_zyanyad_response!(GetSyncStatus);
    impl_into_zyanyad_response!(GetDaaScoreTimestampEstimate);
    impl_into_zyanyad_response!(GetFeeEstimate);
    impl_into_zyanyad_response!(GetFeeEstimateExperimental);
    impl_into_zyanyad_response!(GetCurrentBlockColor);
    impl_into_zyanyad_response!(GetUtxoReturnAddress);
    impl_into_zyanyad_response!(DeployContract);
    impl_into_zyanyad_response!(InvokeContract);
    impl_into_zyanyad_response!(GetContractState);
    impl_into_zyanyad_response!(GetContractCode);
    impl_into_zyanyad_response!(CallContract);

    impl_into_zyanyad_notify_response!(NotifyBlockAdded);
    impl_into_zyanyad_notify_response!(NotifyNewBlockTemplate);
    impl_into_zyanyad_notify_response!(NotifyUtxosChanged);
    impl_into_zyanyad_notify_response!(NotifyPruningPointUtxoSetOverride);
    impl_into_zyanyad_notify_response!(NotifyFinalityConflict);
    impl_into_zyanyad_notify_response!(NotifyVirtualDaaScoreChanged);
    impl_into_zyanyad_notify_response!(NotifyVirtualChainChanged);
    impl_into_zyanyad_notify_response!(NotifySinkBlueScoreChanged);

    impl_into_zyanyad_notify_response!(NotifyUtxosChanged, StopNotifyingUtxosChanged);
    impl_into_zyanyad_notify_response!(NotifyPruningPointUtxoSetOverride, StopNotifyingPruningPointUtxoSetOverride);

    macro_rules! impl_into_zyanyad_response {
        ($name:tt) => {
            paste::paste! {
                impl_into_zyanyad_response_ex!(zyanya_rpc_core::[<$name Response>],[<$name ResponseMessage>],[<$name Response>]);
            }
        };
        ($core_name:tt, $protowire_name:tt) => {
            paste::paste! {
                impl_into_zyanyad_response_base!(zyanya_rpc_core::[<$core_name Response>],[<$protowire_name ResponseMessage>],[<$protowire_name Response>]);
            }
        };
    }
    use impl_into_zyanyad_response;

    macro_rules! impl_into_zyanyad_response_base {
        ($core_struct:path, $protowire_struct:ident, $variant:ident) => {
            // ----------------------------------------------------------------------------
            // rpc_core to protowire
            // ----------------------------------------------------------------------------

            impl From<RpcResult<$core_struct>> for $protowire_struct {
                fn from(item: RpcResult<$core_struct>) -> Self {
                    item.as_ref().map_err(|x| (*x).clone()).into()
                }
            }

            impl From<RpcError> for $protowire_struct {
                fn from(item: RpcError) -> Self {
                    let x: RpcResult<&$core_struct> = Err(item);
                    x.into()
                }
            }

            impl From<$protowire_struct> for zyanyad_response::Payload {
                fn from(item: $protowire_struct) -> Self {
                    zyanyad_response::Payload::$variant(item)
                }
            }

            impl From<$protowire_struct> for ZyanyadResponse {
                fn from(item: $protowire_struct) -> Self {
                    Self { id: 0, payload: Some(zyanyad_response::Payload::$variant(item)) }
                }
            }
        };
    }
    use impl_into_zyanyad_response_base;

    macro_rules! impl_into_zyanyad_response_ex {
        ($core_struct:path, $protowire_struct:ident, $variant:ident) => {
            // ----------------------------------------------------------------------------
            // rpc_core to protowire
            // ----------------------------------------------------------------------------

            impl From<RpcResult<&$core_struct>> for zyanyad_response::Payload {
                fn from(item: RpcResult<&$core_struct>) -> Self {
                    zyanyad_response::Payload::$variant(item.into())
                }
            }

            impl From<RpcResult<&$core_struct>> for ZyanyadResponse {
                fn from(item: RpcResult<&$core_struct>) -> Self {
                    Self { id: 0, payload: Some(item.into()) }
                }
            }

            impl From<RpcResult<$core_struct>> for zyanyad_response::Payload {
                fn from(item: RpcResult<$core_struct>) -> Self {
                    zyanyad_response::Payload::$variant(item.into())
                }
            }

            impl From<RpcResult<$core_struct>> for ZyanyadResponse {
                fn from(item: RpcResult<$core_struct>) -> Self {
                    Self { id: 0, payload: Some(item.into()) }
                }
            }

            impl_into_zyanyad_response_base!($core_struct, $protowire_struct, $variant);

            // ----------------------------------------------------------------------------
            // protowire to rpc_core
            // ----------------------------------------------------------------------------

            impl TryFrom<&zyanyad_response::Payload> for $core_struct {
                type Error = RpcError;
                fn try_from(item: &zyanyad_response::Payload) -> RpcResult<Self> {
                    if let zyanyad_response::Payload::$variant(response) = item {
                        response.try_into()
                    } else {
                        Err(RpcError::MissingRpcFieldError("Payload".to_string(), stringify!($variant).to_string()))
                    }
                }
            }

            impl TryFrom<&ZyanyadResponse> for $core_struct {
                type Error = RpcError;
                fn try_from(item: &ZyanyadResponse) -> RpcResult<Self> {
                    item.payload
                        .as_ref()
                        .ok_or(RpcError::MissingRpcFieldError("ZyanyaResponse".to_string(), "Payload".to_string()))?
                        .try_into()
                }
            }
        };
    }
    use impl_into_zyanyad_response_ex;

    macro_rules! impl_into_zyanyad_notify_response {
        ($name:tt) => {
            impl_into_zyanyad_response!($name);

            paste::paste! {
                impl_into_zyanyad_notify_response_ex!(zyanya_rpc_core::[<$name Response>],[<$name ResponseMessage>]);
            }
        };
        ($core_name:tt, $protowire_name:tt) => {
            impl_into_zyanyad_response!($core_name, $protowire_name);

            paste::paste! {
                impl_into_zyanyad_notify_response_ex!(zyanya_rpc_core::[<$core_name Response>],[<$protowire_name ResponseMessage>]);
            }
        };
    }
    use impl_into_zyanyad_notify_response;

    macro_rules! impl_into_zyanyad_notify_response_ex {
        ($($core_struct:ident)::+, $protowire_struct:ident) => {
            // ----------------------------------------------------------------------------
            // rpc_core to protowire
            // ----------------------------------------------------------------------------

            impl<T> From<Result<(), T>> for $protowire_struct
            where
                T: Into<RpcError>,
            {
                fn from(item: Result<(), T>) -> Self {
                    item
                        .map(|_| $($core_struct)::+{})
                        .map_err(|err| err.into()).into()
                }
            }

        };
    }
    use impl_into_zyanyad_notify_response_ex;
}
