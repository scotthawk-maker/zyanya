"""Data models for Zyanya GhostDAG RPC responses."""

from dataclasses import dataclass, field
from typing import List, Optional


@dataclass
class NodeInfo:
    version: str
    p2p_id: str
    is_synced: bool
    server_version: str
    has_utxo_index: bool
    network: str = "mainnet"


@dataclass
class BlockDagInfo:
    network: str
    block_count: int
    header_count: int
    tip_hashes: List[str]
    difficulty: float
    past_median_time: int
    virtual_parent_hashes: List[str]
    virtual_daa_score: int
    pruning_point_hash: Optional[str] = None


@dataclass
class PeerInfo:
    id: str
    address: str
    last_ping: int
    is_outbound: bool
    time_offset: int
    user_agent: str
    advertised_protocol_version: int


@dataclass
class BalanceInfo:
    address: str
    balance_sompi: int
    balance_zyan: float
    unconfirmed_sompi: int = 0
    unconfirmed_zyan: float = 0.0


@dataclass
class UtxoEntry:
    transaction_id: str
    index: int
    amount_sompi: int
    amount_zyan: float
    script_public_key: str
    block_daa_score: int
    is_coinbase: bool


@dataclass
class MiningTargetInfo:
    target_hex: str
    difficulty: float
    bits: int
    estimated_hashrate: float = 0.0


@dataclass
class Transaction:
    version: int = 0
    inputs: List[dict] = field(default_factory=list)
    outputs: List[dict] = field(default_factory=list)
    lock_time: int = 0
    subnetwork_id: str = "0000000000000000000000000000000000000000"
    gas: int = 0
    payload: str = ""
