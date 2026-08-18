use crate::pb::zyanyad_message::Payload as ZyanyadMessagePayload;

#[repr(u8)]
#[derive(Debug, Copy, Clone, Eq, Hash, PartialEq)]
pub enum ZyanyadMessagePayloadType {
    Addresses = 0,
    Block,
    Transaction,
    BlockLocator,
    RequestAddresses,
    RequestRelayBlocks,
    RequestTransactions,
    IbdBlock,
    InvRelayBlock,
    InvTransactions,
    Ping,
    Pong,
    Verack,
    Version,
    TransactionNotFound,
    Reject,
    PruningPointUtxoSetChunk,
    RequestIbdBlocks,
    UnexpectedPruningPoint,
    IbdBlockLocator,
    IbdBlockLocatorHighestHash,
    RequestNextPruningPointUtxoSetChunk,
    DonePruningPointUtxoSetChunks,
    IbdBlockLocatorHighestHashNotFound,
    BlockWithTrustedData,
    DoneBlocksWithTrustedData,
    RequestPruningPointAndItsAnticone,
    BlockHeaders,
    RequestNextHeaders,
    DoneHeaders,
    RequestPruningPointUtxoSet,
    RequestHeaders,
    RequestBlockLocator,
    PruningPoints,
    RequestPruningPointProof,
    PruningPointProof,
    Ready,
    BlockWithTrustedDataV4,
    TrustedData,
    RequestIbdChainBlockLocator,
    IbdChainBlockLocator,
    RequestAntipast,
    RequestNextPruningPointAndItsAnticoneBlocks,
}

impl From<&ZyanyadMessagePayload> for ZyanyadMessagePayloadType {
    fn from(payload: &ZyanyadMessagePayload) -> Self {
        match payload {
            ZyanyadMessagePayload::Addresses(_) => ZyanyadMessagePayloadType::Addresses,
            ZyanyadMessagePayload::Block(_) => ZyanyadMessagePayloadType::Block,
            ZyanyadMessagePayload::Transaction(_) => ZyanyadMessagePayloadType::Transaction,
            ZyanyadMessagePayload::BlockLocator(_) => ZyanyadMessagePayloadType::BlockLocator,
            ZyanyadMessagePayload::RequestAddresses(_) => ZyanyadMessagePayloadType::RequestAddresses,
            ZyanyadMessagePayload::RequestRelayBlocks(_) => ZyanyadMessagePayloadType::RequestRelayBlocks,
            ZyanyadMessagePayload::RequestTransactions(_) => ZyanyadMessagePayloadType::RequestTransactions,
            ZyanyadMessagePayload::IbdBlock(_) => ZyanyadMessagePayloadType::IbdBlock,
            ZyanyadMessagePayload::InvRelayBlock(_) => ZyanyadMessagePayloadType::InvRelayBlock,
            ZyanyadMessagePayload::InvTransactions(_) => ZyanyadMessagePayloadType::InvTransactions,
            ZyanyadMessagePayload::Ping(_) => ZyanyadMessagePayloadType::Ping,
            ZyanyadMessagePayload::Pong(_) => ZyanyadMessagePayloadType::Pong,
            ZyanyadMessagePayload::Verack(_) => ZyanyadMessagePayloadType::Verack,
            ZyanyadMessagePayload::Version(_) => ZyanyadMessagePayloadType::Version,
            ZyanyadMessagePayload::TransactionNotFound(_) => ZyanyadMessagePayloadType::TransactionNotFound,
            ZyanyadMessagePayload::Reject(_) => ZyanyadMessagePayloadType::Reject,
            ZyanyadMessagePayload::PruningPointUtxoSetChunk(_) => ZyanyadMessagePayloadType::PruningPointUtxoSetChunk,
            ZyanyadMessagePayload::RequestIbdBlocks(_) => ZyanyadMessagePayloadType::RequestIbdBlocks,
            ZyanyadMessagePayload::UnexpectedPruningPoint(_) => ZyanyadMessagePayloadType::UnexpectedPruningPoint,
            ZyanyadMessagePayload::IbdBlockLocator(_) => ZyanyadMessagePayloadType::IbdBlockLocator,
            ZyanyadMessagePayload::IbdBlockLocatorHighestHash(_) => ZyanyadMessagePayloadType::IbdBlockLocatorHighestHash,
            ZyanyadMessagePayload::RequestNextPruningPointUtxoSetChunk(_) => {
                ZyanyadMessagePayloadType::RequestNextPruningPointUtxoSetChunk
            }
            ZyanyadMessagePayload::DonePruningPointUtxoSetChunks(_) => ZyanyadMessagePayloadType::DonePruningPointUtxoSetChunks,
            ZyanyadMessagePayload::IbdBlockLocatorHighestHashNotFound(_) => {
                ZyanyadMessagePayloadType::IbdBlockLocatorHighestHashNotFound
            }
            ZyanyadMessagePayload::BlockWithTrustedData(_) => ZyanyadMessagePayloadType::BlockWithTrustedData,
            ZyanyadMessagePayload::DoneBlocksWithTrustedData(_) => ZyanyadMessagePayloadType::DoneBlocksWithTrustedData,
            ZyanyadMessagePayload::RequestPruningPointAndItsAnticone(_) => {
                ZyanyadMessagePayloadType::RequestPruningPointAndItsAnticone
            }
            ZyanyadMessagePayload::BlockHeaders(_) => ZyanyadMessagePayloadType::BlockHeaders,
            ZyanyadMessagePayload::RequestNextHeaders(_) => ZyanyadMessagePayloadType::RequestNextHeaders,
            ZyanyadMessagePayload::DoneHeaders(_) => ZyanyadMessagePayloadType::DoneHeaders,
            ZyanyadMessagePayload::RequestPruningPointUtxoSet(_) => ZyanyadMessagePayloadType::RequestPruningPointUtxoSet,
            ZyanyadMessagePayload::RequestHeaders(_) => ZyanyadMessagePayloadType::RequestHeaders,
            ZyanyadMessagePayload::RequestBlockLocator(_) => ZyanyadMessagePayloadType::RequestBlockLocator,
            ZyanyadMessagePayload::PruningPoints(_) => ZyanyadMessagePayloadType::PruningPoints,
            ZyanyadMessagePayload::RequestPruningPointProof(_) => ZyanyadMessagePayloadType::RequestPruningPointProof,
            ZyanyadMessagePayload::PruningPointProof(_) => ZyanyadMessagePayloadType::PruningPointProof,
            ZyanyadMessagePayload::Ready(_) => ZyanyadMessagePayloadType::Ready,
            ZyanyadMessagePayload::BlockWithTrustedDataV4(_) => ZyanyadMessagePayloadType::BlockWithTrustedDataV4,
            ZyanyadMessagePayload::TrustedData(_) => ZyanyadMessagePayloadType::TrustedData,
            ZyanyadMessagePayload::RequestIbdChainBlockLocator(_) => ZyanyadMessagePayloadType::RequestIbdChainBlockLocator,
            ZyanyadMessagePayload::IbdChainBlockLocator(_) => ZyanyadMessagePayloadType::IbdChainBlockLocator,
            ZyanyadMessagePayload::RequestAntipast(_) => ZyanyadMessagePayloadType::RequestAntipast,
            ZyanyadMessagePayload::RequestNextPruningPointAndItsAnticoneBlocks(_) => {
                ZyanyadMessagePayloadType::RequestNextPruningPointAndItsAnticoneBlocks
            }
        }
    }
}
