# zyanya-py

Async Python 3.9+ Client Library for the **Zyanya GhostDAG L1** blockchain.

## Features
- 🚀 **Asynchronous First**: Built on `asyncio` for high-throughput node interaction.
- 🦅 **GhostDAG Metrics**: Query live DAA scores, tip hashes, difficulty, and block counts.
- 💼 **Wallet & Balance Queries**: Fetch confirmed sompi/ZYAN balances and live UTXO sets.
- ⛏️ **Mining Target Estimation**: Compute AstroBWTv3 target compact bits and difficulty.
- 🤖 **Agentic & AI-Ready**: Standard Python typing with full IDE autocompletion for trading bots and WebMCP agents.

## Installation
```bash
pip install -e .
```

## Quickstart
```python
import asyncio
from zyanya import ZyanyaClient, ZyanyaNetwork

async def main():
    async with ZyanyaClient(endpoint="http://127.0.0.1:18110") as client:
        # Get Node Status
        info = await client.get_info()
        print(f"Node Version: {info.version} (Synced: {info.is_synced})")

        # Get DAG Metrics
        dag = await client.get_block_dag_info()
        print(f"DAG Height: {dag.block_count} | DAA Score: {dag.virtual_daa_score}")
        print(f"Current Difficulty: {dag.difficulty:.4f}")

        # Query Balance
        balance = await client.get_balance("zyanya:qq...")
        print(f"Balance: {balance.balance_zyan} ZYAN ({balance.balance_sompi} sompi)")

asyncio.run(main())
```
