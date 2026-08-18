# Architecture

## Crate Structure

```
zyanya/
├── consensus/           # GhostDAG consensus engine
│   ├── core/           # TX types, coinbase, config, genesis
│   └── src/            # Consensus pipeline, virtual processor, contract processor
├── zyanya-vm/          # Smart contract VM
│   ├── src/            # VM, opcodes, stack, memory, gas, state
│   ├── tests/          # Integration tests (staking, graduation, compiler)
│   └── compiler/       # ZCL lexer, parser, codegen
├── zyanya-wallet/      # CLI + TUI wallet
│   └── src/            # Key management, wallet ops, TUI
├── zyanya-explorer/    # Block explorer + web UI
│   ├── src/            # API handlers, RPC client, web pages
│   └── assets/         # Brand SVGs, icons
├── database/           # RocksDB storage layer
├── rpc/                # gRPC + wRPC server/client
├── crypto/             # Hashes, addresses, BIP-32
├── daemon/             # zyanyad node daemon
├── wasm/               # WebAssembly bindings
├── mining/             # Mining/pow
├── components/         # P2P, connection manager, address manager
└── utils/              # Shared utilities (fd budget, hex, mem size)
```

## Consensus

- **GhostDAG** — blocks reference multiple parents; topological sort determines ordering
- **Virtual processor** — resolves the virtual block (merged block of all anticone) in topological order
- **Contract processor** — executes smart contract TXs (`Deploy` / `Invoke`) in GhostDAG order
- **State cache** — `ContractStateCache` (BTreeMap for deterministic iteration) holds in-memory state during block execution, committed atomically to RocksDB on virtual block acceptance
- **Reorg handling** — contract state changes roll back on reorgs (incremental state diffs per block height)

## Smart Contract Execution Flow

```
TX (Deploy/Invoke) → ContractProcessor.process_contract_tx()
  → VM.execute_stateful(code, contract_address, state_cache)
    → Opcode dispatch loop with gas metering
    → SLOAD/SSTORE against ContractStateCache
    → Return value + gas used
  → ContractExecutionOutcome (success/fail, gas, fees)
  → State changes remain in cache, committed on virtual block acceptance
```

## Key Design Decisions

1. **BTreeMap for state cache** — deterministic iteration order prevents consensus forks (AUDIT HIGH-01)
2. **Checked arithmetic in VM** — all math opcodes return `ArithmeticOverflow` error instead of silently wrapping (AUDIT HIGH-02)
3. **Protocol-level AMM** — bonding curve + AMM implemented as a smart contract, not native consensus code
4. **Non-custodial token deploy** — users sign TXs client-side; explorer only relays unsigned TX building
5. **IPv6-only explorer** — `socket2` with `IPV6_V6ONLY=true` enforces IPv6 positioning