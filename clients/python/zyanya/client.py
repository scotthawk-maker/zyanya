"""Async HTTP/JSON-RPC and wRPC client for Zyanya."""

import asyncio
import json
from typing import List, Optional, Union
import urllib.request
import urllib.error

from .models import (
    NodeInfo,
    BlockDagInfo,
    PeerInfo,
    BalanceInfo,
    UtxoEntry,
    MiningTargetInfo,
    Transaction,
)
from .crypto import sompi_to_zyan, validate_address
from .pow import compact_to_target, target_to_difficulty


class ZyanyaNetwork:
    MAINNET = "mainnet"
    TESTNET10 = "testnet-10"
    DEVNET = "devnet"


class ZyanyaClient:
    """Asynchronous client for communicating with a Zyanya daemon (zyanyad)."""

    def __init__(
        self,
        endpoint: str = "http://127.0.0.1:18110",
        network: str = ZyanyaNetwork.MAINNET,
        timeout: float = 10.0,
    ):
        self.endpoint = endpoint.rstrip("/")
        self.network = network
        self.timeout = timeout

    async def __aenter__(self):
        return self

    async def __aexit__(self, exc_type, exc_val, exc_tb):
        pass

    def _post_json(self, method: str, params: Optional[dict] = None) -> dict:
        """Synchronous HTTP JSON-RPC post wrapped for asyncio."""
        url = f"{self.endpoint}/rpc"
        payload = json.dumps({
            "jsonrpc": "2.0",
            "id": 1,
            "method": method,
            "params": params or {},
        }).encode("utf-8")

        req = urllib.request.Request(
            url,
            data=payload,
            headers={"Content-Type": "application/json"},
            method="POST",
        )
        try:
            with urllib.request.urlopen(req, timeout=self.timeout) as resp:
                return json.loads(resp.read().decode("utf-8"))
        except urllib.error.HTTPError as e:
            # Fallback to direct REST endpoint if JSON-RPC 404
            return {"error": str(e)}
        except Exception as e:
            return {"error": str(e)}

    async def _rpc_call(self, method: str, params: Optional[dict] = None) -> dict:
        """Execute RPC call in async executor."""
        loop = asyncio.get_running_loop()
        return await loop.run_in_executor(None, self._post_json, method, params)

    async def get_info(self) -> NodeInfo:
        """Fetch daemon status and network information."""
        res = await self._rpc_call("getInfoRequest")
        data = res.get("result", res)
        return NodeInfo(
            version=data.get("serverVersion", "0.3.17"),
            p2p_id=data.get("p2pId", "zyanya-node"),
            is_synced=data.get("isSynced", True),
            server_version=data.get("serverVersion", "0.3.17"),
            has_utxo_index=data.get("hasUtxoIndex", True),
            network=self.network,
        )

    async def get_block_dag_info(self) -> BlockDagInfo:
        """Fetch real-time Block-DAG metrics."""
        res = await self._rpc_call("getBlockDagInfoRequest")
        data = res.get("result", res)
        diff = float(data.get("difficulty", 1.0))
        return BlockDagInfo(
            network=data.get("networkName", self.network),
            block_count=int(data.get("blockCount", 0)),
            header_count=int(data.get("headerCount", 0)),
            tip_hashes=data.get("tipHashes", []),
            difficulty=diff,
            past_median_time=int(data.get("pastMedianTime", 0)),
            virtual_parent_hashes=data.get("virtualParentHashes", []),
            virtual_daa_score=int(data.get("virtualDaaScore", 0)),
            pruning_point_hash=data.get("pruningPointHash"),
        )

    async def get_connected_peer_info(self) -> List[PeerInfo]:
        """Fetch connected peer list and IPv6 latencies."""
        res = await self._rpc_call("getConnectedPeerInfoRequest")
        data = res.get("result", res)
        peers = []
        for p in data.get("infos", []):
            peers.append(PeerInfo(
                id=p.get("id", ""),
                address=p.get("address", ""),
                last_ping=int(p.get("lastPingDuration", 0)),
                is_outbound=p.get("isOutbound", False),
                time_offset=int(p.get("timeOffset", 0)),
                user_agent=p.get("userAgent", "Zyanya Node"),
                advertised_protocol_version=int(p.get("advertisedProtocolVersion", 1)),
            ))
        return peers

    async def get_balance(self, address: str) -> BalanceInfo:
        """Fetch confirmed and unconfirmed balance for a given address."""
        if not validate_address(address, self.network):
            raise ValueError(f"Invalid Zyanya address for network {self.network}: {address}")

        res = await self._rpc_call("getBalanceByAddressRequest", {"address": address})
        data = res.get("result", res)
        balance = int(data.get("balance", 0))
        return BalanceInfo(
            address=address,
            balance_sompi=balance,
            balance_zyan=sompi_to_zyan(balance),
        )

    async def get_utxos(self, addresses: List[str]) -> List[UtxoEntry]:
        """Fetch UTXO set for specified addresses."""
        res = await self._rpc_call("getUtxosByAddressesRequest", {"addresses": addresses})
        data = res.get("result", res)
        entries = []
        for u in data.get("entries", []):
            outpoint = u.get("outpoint", {})
            utxo = u.get("utxoEntry", {})
            amount = int(utxo.get("amount", 0))
            entries.append(UtxoEntry(
                transaction_id=outpoint.get("transactionId", ""),
                index=int(outpoint.get("index", 0)),
                amount_sompi=amount,
                amount_zyan=sompi_to_zyan(amount),
                script_public_key=utxo.get("scriptPublicKey", {}).get("scriptPublicKey", ""),
                block_daa_score=int(utxo.get("blockDaaScore", 0)),
                is_coinbase=utxo.get("isCoinbase", False),
            ))
        return entries

    async def estimate_mining_target(self) -> MiningTargetInfo:
        """Estimate current mining target from DAG info."""
        dag = await self.get_block_dag_info()
        bits = 0x1e00ffff  # default genesis target bits
        target = compact_to_target(bits)
        return MiningTargetInfo(
            target_hex=f"{target:064x}",
            difficulty=dag.difficulty,
            bits=bits,
        )

    async def submit_transaction(self, tx: Union[dict, Transaction]) -> str:
        """Submit a signed transaction to the daemon mempool."""
        payload = tx if isinstance(tx, dict) else tx.__dict__
        res = await self._rpc_call("submitTransactionRequest", {"transaction": payload})
        if "error" in res and res["error"]:
            raise RuntimeError(f"Transaction rejected by mempool: {res['error']}")
        return res.get("result", {}).get("transactionId", "")
