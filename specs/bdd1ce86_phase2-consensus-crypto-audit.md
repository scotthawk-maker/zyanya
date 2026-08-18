# Phase 2 Security Audit Plan — Consensus + Crypto

## Objective

Deep, line-level security audit of the Zyanya consensus and cryptographic modules
(mostly upstream Kaspa/rusty-spectre code, renamed to `zyanya-*` crates). The goal is to:

1. Identify and verify **Zyanya-specific modifications** to upstream consensus code
   (these are the highest-risk items — upstream code is battle-tested, forks are not).
2. Check the upstream modules for **inherent vulnerabilities** in the listed categories
   (integer overflow, consensus bypass, PoW/difficulty manipulation, hash correctness,
   Muhash correctness, address encoding, script VM safety, unsafe blocks).
3. Write a single findings report to:

```
audit_reports/phase2_findings.md
```

The report must contain, for **every** finding:

- **Severity**: `CRITICAL` / `HIGH` / `MEDIUM` / `LOW`
- **Location**: `file:line` (exact line numbers, verified against the current tree)
- **Description**: what the bug is and how it is triggered
- **Code snippet**: the offending lines (verbatim, trimmed)
- **Recommended fix**: concrete, minimal remediation

Order findings by severity (CRITICAL first), then by file path.

---

## How to tell Zyanya code from upstream Kaspa code

The git history is only 3 commits (initial import + audit scaffolding), so there is no
upstream diff to consult. Identify Zyanya modifications by:

- `grep -rin "zyanya"` — mostly crate renames (`zyanya-consensus-core`, etc.) and network
  prefixes (`zyanya:`, `zyanyatest:`). These are cosmetic, not security-relevant.
- **Substantive Zyanya changes** found so far (verify each):
  1. `consensus/core/src/config/params.rs` — new consensus fields
     `deflationary_phase_daa_score`, `pre_deflationary_phase_base_subsidy`; changed
     `net_magic` (`"ZYAN"`, `"ZYNT"`, `"T11N"`, `"SIMN"`, `"DEVN"`); DNS seeders renamed.
  2. `consensus/src/processes/coinbase.rs` — **coinbase vesting split** (50% liquid +
     50% vested over 12 monthly CSV-locked outputs) and **deflationary subsidy schedule**
     (mainnet uses a smooth geometric decay computed with `f64::powf`).
  3. `consensus/core/src/subnets.rs` — new `SUBNETWORK_ID_SMART_CONTRACT` (byte `3`).
  4. `consensus/core/src/tx.rs` — new `DeployContractPayload` / `InvokeContractPayload` /
     `ContractPayload` (Borsh-serialized) and the `payload` field semantics.
  5. `consensus/src/processes/transaction_validator/tx_validation_in_isolation.rs` —
     `check_contract_payload_in_isolation` and the coinbase output-limit change (`* 13`).
  6. `consensus/src/model/stores/contract.rs` — contract state store (already covered in
     Phase 1, but re-check the consensus-facing surface: custody accounting, commit-on-failure).

Everything else is expected to be upstream Kaspa; audit it for inherent bugs, not diffs.

---

## Audit scope (files, in priority order)

### CRITICAL tier — Zyanya consensus modifications

| # | File | Vulnerability categories |
|---|------|--------------------------|
| 1 | `consensus/src/processes/coinbase.rs` | Subsidy schedule (f64 non-determinism), vesting split correctness, coinbase output-count limit mismatch, unchecked `red_reward` addition, `Vec::with_capacity` overflow |
| 2 | `consensus/src/processes/transaction_validator/tx_validation_in_isolation.rs` | Coinbase output limit `(ghostdag_k+2)*13`, contract payload validation gaps (no size/length bounds), `check_transaction_outputs_count` error-field bug |
| 3 | `consensus/core/src/config/params.rs` | `max_tx_inputs/outputs = 1_000_000_000` (mass overflow surface), deflationary params, `net_magic` isolation, fork activation values |
| 4 | `consensus/core/src/tx.rs` | `ContractPayload` Borsh deserialization (untrusted input), `TransactionMass` atomic, tx size/mass estimation |
| 5 | `consensus/core/src/subnets.rs` | `SUBNETWORK_ID_SMART_CONTRACT` classification, `is_builtin`/`is_native` gating |

### CRITICAL tier — consensus-critical crypto

| # | File | Vulnerability categories |
|---|------|--------------------------|
| 6 | `crypto/muhash/src/u3072.rs` + `crypto/muhash/src/lib.rs` | Modular arithmetic correctness (`is_overflow`, `full_reduce`, `mul`, `div`, `inverse`), `assert!` invariants that can panic, serialization/deserialization overflow rejection |
| 7 | `crypto/txscript/src/opcodes/mod.rs` + `crypto/txscript/src/data_stack.rs` + `crypto/txscript/src/lib.rs` | Script VM opcode safety, stack depth, sig-op counting, numeric encoding (minimal-data), multisig, conditional execution |
| 8 | `crypto/addresses/src/bech32.rs` + `crypto/addresses/src/lib.rs` | Bech32 decode (charset bounds, checksum), payload length/version validation, prefix handling |
| 9 | `crypto/hashes/src/hashers.rs` + `pow_hashers.rs` + `lib.rs` | Domain separation correctness, hash length, cSHAKE/Blake2b/SHA256 usage |
| 10 | `crypto/merkle/src/lib.rs` | Merkle root correctness, odd/even padding, `ZERO_HASH` handling |

### HIGH tier — PoW, difficulty, DAG

| # | File | Vulnerability categories |
|---|------|--------------------------|
| 11 | `consensus/pow/src/lib.rs` + `matrix.rs` + `xoshiro.rs` | PoW target comparison, block-level calculation, matrix rank (f64), `MaybeUninit` unsafe block, heavy-hash correctness |
| 12 | `consensus/src/processes/difficulty.rs` | Difficulty target calculation (Uint320 overflow avoidance), DAA score, `calc_work`, `level_work` |
| 13 | `consensus/src/processes/ghostdag/protocol.rs` + `ordering.rs` + `mergeset.rs` | GhostDAG/Spectre coloring correctness, blue-score/blue-work accumulation, k-cluster checks |
| 14 | `consensus/src/pipeline/header_processor/` (`pre_pow_validation.rs`, `post_pow_validation.rs`, `pre_ghostdag_validation.rs`, `processor.rs`) | Block acceptance bypass, timestamp/median-time, difficulty check, PoW check ordering |
| 15 | `consensus/core/src/mass/mod.rs` | Transaction mass calculation overflow (compute mass uses plain `*`/`+`), storage mass (KIP-0009) |
| 16 | `consensus/core/src/hashing/sighash.rs` + `sighash_type.rs` | Signature hash correctness, sighash type validation, ECDSA vs Schnorr |

### MEDIUM tier — supporting consensus logic

| # | File | Vulnerability categories |
|---|------|--------------------------|
| 17 | `consensus/src/processes/transaction_validator/tx_validation_in_utxo_context.rs` + `tx_validation_in_header_context.rs` | UTXO-context validation, sequence lock, coinbase maturity, fee/subsidy accounting |
| 18 | `consensus/src/processes/past_median_time.rs` + `window.rs` | Timestamp median calculation, window sampling |
| 19 | `consensus/src/processes/reachability/` | DAG reachability, interval/tree correctness, reindex |
| 20 | `consensus/src/processes/pruning_proof/` | Pruning proof build/validate/apply |
| 21 | `consensus/src/model/stores/` (ghostdag, utxo, headers, relations, reachability) | Store invariants, serialization safety |
| 22 | `consensus/client/src/` + `consensus/wasm/src/` + `consensus/notify/src/` | Client-side validation, WASM bindings, notification service (lower priority) |
| 23 | `math/src/uint.rs` + `math/src/lib.rs` | `Uint256`/`Uint3072`/`Uint320` arithmetic, `from_compact_target_bits`/`compact_target_bits`, `debug_assert!` overflow checks in operator impls |

---

## Findings to validate and expand

For each item: confirm the exact `file:line`, reproduce the trigger mentally (or with a
minimal test where feasible), and write it up in the report format. Items marked **[NEW]**
were not in the Phase 1 report and are specific to this phase.

### 1. Coinbase output-count limit is too small for the 13× vesting split — CRITICAL [NEW]

- **File**: `consensus/src/processes/transaction_validator/tx_validation_in_isolation.rs:46-48`
  and `consensus/src/processes/coinbase.rs:125-190`
- **What to verify**:
  - `check_coinbase_in_isolation` enforces `outputs_limit = (ghostdag_k + 2) * 13`
    (mainnet: `(18+2)*13 = 260`).
  - `expected_coinbase_transaction` emits **13 outputs per DAA-window blue block**
    (1 liquid + 12 vested) plus up to 13 for the red reward.
  - The number of DAA-window blues in a mergeset is bounded by the **DAA/difficulty
    window** (mainnet legacy `LEGACY_DIFFICULTY_WINDOW_SIZE = 2641`) and by
    `mergeset_size_limit = k*10 = 180` — **not** by `ghostdag_k`.
  - Therefore a block whose mergeset contains > ~19 DAA-window blues produces a coinbase
    with > 260 outputs and is **rejected as invalid** by every node → miner loses the
    block, and if the network reaches that state, no valid block can be produced
    (chain halt / consensus split).
  - Compare against upstream Kaspa's coinbase output limit to confirm the correct bound;
    the `* 13` factor is clearly Zyanya and the `ghostdag_k + 2` base was not re-derived
    for the vesting split.
- **Fix direction**: derive the limit from the actual maximum number of payable blues
  (e.g. `(mergeset_size_limit + 1) * 13` or the DAA-window bound), or cap the number of
  vested outputs per blue; add a test that builds a coinbase with > 19 DAA-window blues.

### 2. Floating-point (`f64::powf`) in consensus-critical subsidy schedule — CRITICAL [NEW]

- **File**: `consensus/src/processes/coinbase.rs:88-95` (mainnet table build in
  `CoinbaseManager::new`), consumed by `calc_block_subsidy` at `:276-289`
- **What to verify**:
  - For mainnet (`deflationary_phase_daa_score == 31_449_600`) the 727-entry subsidy
    table is computed as `(initial * (1.0 - decay_rate).powf(m)).round() as u64` with
    `decay_rate = 0.004847033515f64`.
  - `f64::powf` is **not guaranteed bit-identical across platforms/libm** (x86 vs ARM,
    musl vs glibc). A 1-ULP difference can flip `.round()` to a different integer for
    some month index → two nodes compute different subsidy tables → **consensus fork**.
  - Also note the table is computed once at startup, so the divergence is silent and
    persistent per-node.
- **Fix direction**: replace the float schedule with a fixed integer table (like the
  existing `SUBSIDY_BY_MONTH_TABLE` for non-mainnet networks) or an integer recurrence
  (e.g. fixed-point / rational decay) that is exactly reproducible; add cross-platform
  golden-vector tests.

### 3. Coinbase vesting split: verify no value creation/loss and CSV correctness — HIGH [NEW]

- **File**: `consensus/src/processes/coinbase.rs:125-190`
- **What to verify**:
  - `liquid = total/2`, `vested = total - liquid`; `monthly = vested/12`,
    `remainder = vested - monthly*11`; 12 outputs sum to `vested` exactly (11×monthly +
    remainder). Confirm the split never creates or destroys sompi (rounding is
    deterministic and total is preserved).
  - `lock_blocks = (i+1) * blocks_per_month` with `blocks_per_month = SECONDS_PER_MONTH * bps`
    (`:82`). Confirm `SECONDS_PER_MONTH = 2629800` and the CSV lock semantics match the
    `OpCheckSequenceVerify` script built by `create_csv_locked_script` (`:60-70`).
  - Confirm the coinbase maturity (100 blocks) still applies to the liquid output and
    that vested outputs are not spendable before their CSV lock (the Phase 1 test
    `csv_vested_output_spending_test` covers the happy path — check edge cases: sequence
    bit 63 disabled, `u64::MAX` sequence).
  - `red_reward += reward_data.subsidy + reward_data.total_fees` (`:162`) is unchecked
    addition — bounded in practice (≤ mergeset_size_limit × MAX_SOMPI ≈ 5.2e18 < u64::MAX)
    but confirm and consider `checked_add`.
  - `Vec::with_capacity((mergeset_blues.len() + 1) * 13)` (`:125`) — confirm no overflow
    (bounded by mergeset_size_limit) and that it is only a capacity hint.

### 4. Contract payload validation gaps (DoS / consensus surface) — HIGH [NEW]

- **File**: `consensus/src/processes/transaction_validator/tx_validation_in_isolation.rs:157-200`
  and `consensus/core/src/tx.rs:134-170`
- **What to verify**:
  - `check_contract_payload_in_isolation` checks `bytecode` non-empty, `max_gas != 0`,
    `gas_price != 0`, `tx.gas == max_gas` — but **no bound** on `bytecode.len()`,
    `parameters.len()`, or `deposit_amount`.
  - A deploy tx with a multi-GB `bytecode` or an invoke with millions of `parameters`
    (each a `u64`) is a memory/CPU DoS at validation time and at execution time
    (the VM in `zyanya-vm` is out of scope here but the consensus layer must bound the
    payload before it reaches the VM).
  - `ContractPayload::from_slice` uses `borsh::from_slice` on untrusted `tx.payload` —
    confirm Borsh rejects trailing bytes / malformed length prefixes safely (no OOM on a
    huge declared length).
- **Fix direction**: add explicit `max_bytecode_len`, `max_parameters_len`, and
  `deposit_amount` bounds to consensus params and enforce them here; bound `tx.payload`
  length before Borsh decode.

### 5. `check_transaction_outputs_count` reports inputs instead of outputs — LOW [NEW]

- **File**: `consensus/src/processes/transaction_validator/tx_validation_in_isolation.rs:60-63`
- **What to verify**: on `tx.outputs.len() > max_tx_outputs` the error is
  `TooManyOutputs(tx.inputs.len(), self.max_tx_inputs)` — wrong fields (copy-paste).
  Not a consensus bypass (the check still fails), but it corrupts diagnostics and could
  mask the real limit in logs. Confirm and report as LOW.
- **Fix direction**: `TooManyOutputs(tx.outputs.len(), self.max_tx_outputs)`.

### 6. Muhash `U3072` correctness and panic-on-invariant — HIGH [NEW]

- **File**: `crypto/muhash/src/u3072.rs` (`is_overflow` :49, `full_reduce` :78, `mul` :90-155,
  `inverse` :157, `div` :175) and `crypto/muhash/src/lib.rs` (`serialize` :105,
  `deserialize` :111, `normalize` :99)
- **What to verify**:
  - `mul` contains **non-debug** `assert_eq!(carry_highest, 0)` (:123),
    `assert_eq!(carry_high, 0)` (:143), `assert!(carry_low == 0 || carry_low == 1)` (:144).
    If any invariant is ever violated (e.g. via a crafted serialized MuHash), the node
    **panics** in release builds → remote DoS. Confirm these are internal invariants that
    cannot be reached from untrusted input, or downgrade to `debug_assert!`.
  - `deserialize` rejects overflow (`is_overflow`) but confirm the boundary: values in
    `[p, 2^3072)` are rejected, values `< p` accepted; `full_reduce` maps overflow values
    back into the field. Verify `PRIME_DIFF = 1103717` and the prime
    `2^3072 - 1103717` are correct (matches upstream Kaspa).
  - `inverse` uses `Uint3072::mod_inverse` (malachite) — confirm `0` is handled
    (`0/x = 0`) and that `div` by zero cannot panic.
  - Confirm `data_to_element` (ChaCha20 from Blake2b hash) matches upstream exactly
    (consensus-critical: any change forks the UTXO-set hash).
- **Fix direction**: keep as-is if invariants are unreachable from untrusted input;
  otherwise convert to `debug_assert!` and return an error. Add differential tests
  against upstream Kaspa test vectors (already present in `lib.rs` tests — verify they
  are unmodified).

### 7. Script VM opcode safety — HIGH [NEW]

- **File**: `crypto/txscript/src/lib.rs` (`execute_opcode` :318, `execute_script` :340,
  `op_check_multisig_schnorr_or_ecdsa` :465), `crypto/txscript/src/opcodes/mod.rs`
  (arithmetic opcodes :592-700, `deserialize_next_opcode` in `macros.rs`),
  `crypto/txscript/src/data_stack.rs`
- **What to verify**:
  - Limits: `MAX_STACK_SIZE = 244`, `MAX_SCRIPTS_SIZE = 10_000`,
    `MAX_SCRIPT_ELEMENT_SIZE = 520`, `MAX_OPS_PER_SCRIPT = 201`,
    `MAX_PUB_KEYS_PER_MUTLTISIG = 20`. Confirm each is enforced on **every** path
    (including P2SH redeem script and multisig key/sig counts).
  - Arithmetic opcodes use `checked_add/sub/neg/abs` (good) — confirm `OpMul/OpDiv/OpMod/
    OpLShift/OpRShift` are disabled and `Op2Mul/Op2Div` disabled (they are, :594-598).
  - `deserialize_next_opcode` / `opcode_serde!` — confirm malformed push lengths
    (`OpPushData1/2/4` with truncated data) return `MalformedPush` and cannot read OOB
    (the `it.take(length)` + length check in `macros.rs` looks correct — verify).
  - `check_minimal_data_push` and `deserialize_i64` minimal-encoding rules — confirm no
    bypass that lets a non-minimal number be interpreted (consensus rule).
  - Multisig: `num_keys`/`num_sigs` negative and `num_sigs > num_keys` rejected; the
    `NullFail` rule; sig-op counting (`num_ops += num_keys`).
  - `execute_script` resets `num_ops` per script and clears `astack`; confirm the
    conditional stack (`cond_stack`) cannot be left unbalanced across scripts.
- **Fix direction**: report any gap; otherwise document as verified upstream.

### 8. Bech32 address decoding — MEDIUM [NEW]

- **File**: `crypto/addresses/src/bech32.rs` (`decode_payload` :110-140, `REV_CHARSET` :4,
  `polymod` :14, `conv5to8` :77) and `crypto/addresses/src/lib.rs`
- **What to verify**:
  - `REV_CHARSET` is 123 entries; `*REV_CHARSET.get(*b as usize).unwrap_or(&100)` — a
    byte ≥ 123 maps to `100` (invalid) and is rejected. Confirm no byte value can index
    out of bounds (the `get` + `unwrap_or` is safe).
  - `decode_payload` checks `address.len() < 8` and splits checksum; confirm the
    checksum comparison and `conv5to8` padding cannot produce a wrong-length payload
    (e.g. a 5-bit group that decodes to a non-byte-aligned payload).
  - Confirm version byte (`payload_u8[0]`) is validated against known `Version` values
    and payload length matches the version (32-byte Schnorr, 33-byte ECDSA, 32-byte
    script hash) — see `Address::new` in `lib.rs`.
  - Confirm prefix matching is exact (no case/separator confusion) — the tests at
    `lib.rs:616-642` cover invalid chars/prefix/checksum; verify they are unmodified.
- **Fix direction**: report any gap; otherwise document as verified upstream.

### 9. PoW / difficulty manipulation — HIGH [NEW]

- **File**: `consensus/pow/src/lib.rs` (`check_pow` :60, `calc_level_from_pow` :75),
  `consensus/pow/src/matrix.rs` (`compute_rank` :65, `heavy_hash` :95),
  `consensus/src/processes/difficulty.rs` (`calculate_difficulty_bits` :150/:230,
  `calc_work` :270, `level_work` :280), `math/src/lib.rs`
  (`from_compact_target_bits` :64, `compact_target_bits` :87)
- **What to verify**:
  - `check_pow` uses `pow <= target` (inclusive) — confirm this matches upstream
    (off-by-one in target comparison is a classic difficulty-manipulation bug).
  - `from_compact_target_bits` / `compact_target_bits` round-trip and the
    `mant > 0x7FFFFF` negative-mantissa rejection — confirm no way to encode a target
    that is easier than intended (e.g. exponent underflow when `unshifted_expt <= 3`).
  - `calculate_difficulty_bits` uses `Uint320` for the sum and `max(max_ts - min_ts, 1)`
    to avoid div-by-zero; confirm the sampled vs full window formulas match upstream
    (the `* difficulty_sample_rate` difference is intentional).
  - `compute_rank` uses `f64` Gaussian elimination with `EPS = 1e-9` — this is upstream
    behavior but confirm it is deterministic (matrix entries are small integers 0..15,
    so the float rank computation is exact in practice; note any risk).
  - `heavy_hash` uses `unsafe { MaybeUninit::uninit().assume_init() }` then writes all
    64 entries — confirm the write loop covers every index (it does: `2*i` and `2*i+1`
    for i in 0..32) and the `transmute` is sound.
- **Fix direction**: report any deviation from upstream; otherwise document as verified.

### 10. GhostDAG / Spectre coloring correctness — HIGH [NEW]

- **File**: `consensus/src/processes/ghostdag/protocol.rs` (`ghostdag` :110,
  `check_blue_candidate` :230, `check_blue_candidate_with_chain_block` :180,
  `blue_anticone_size` :210), `ordering.rs`, `mergeset.rs`
- **What to verify**:
  - Blue-score accumulation (`blue_score = sp_blue_score + mergeset_blues.len()`) and
    blue-work accumulation (`calc_work(bits).max(level_work)` summed) — confirm no
    overflow (BlueWorkType is Uint192; `calc_work` asserts ≤ 2^192).
  - k-cluster checks: `candidate_blue_anticone_size > k` and
    `blues_anticone_sizes == k` rejections; the `assert!(... <= k)` sanity check at
    `:220` is a **non-debug** assert — confirm it cannot be triggered by a malicious
    DAG (else remote panic/DoS).
  - `find_selected_parent` uses `.max()` on blue work — confirm tie-breaking is
    deterministic (hash ordering) and matches upstream.
  - `ordered_mergeset_without_selected_parent` / mergeset ordering — confirm the
    topological order is consensus-agreed and cannot be influenced to change coloring.
- **Fix direction**: report any deviation; otherwise document as verified upstream.

### 11. Transaction mass calculation overflow — MEDIUM [NEW]

- **File**: `consensus/core/src/mass/mod.rs` (`calc_tx_compute_mass` :87-105,
  `transaction_estimated_serialized_size` :14-31, `calc_storage_mass` :134-190)
- **What to verify**:
  - `calc_tx_compute_mass` uses plain `*` and `+` (`size * mass_per_tx_byte`,
    `total_script_public_key_size * mass_per_script_pub_key_byte`,
    `total_sigops * mass_per_sig_op`). With mainnet `max_tx_inputs/outputs = 1e9` and
    `max_signature_script_len = 1e9`, a single tx's estimated size can be ~1e18 bytes →
    `size * mass_per_tx_byte` overflows u64 in release (silent wrap) → mass underflow →
    **block-mass limit bypass**.
  - In practice the p2p layer caps message size at 1 GB (see the comment in
    `params.rs:366-369`), but confirm the consensus path itself does not rely solely on
    that and that `max_block_mass = 500_000` is enforced **after** mass is computed
    (a wrapped mass could be < 500_000 and pass).
  - `calc_storage_mass` uses `checked_add`/`saturating_*` (good) — confirm the
    `storage_mass_parameter / out` division cannot divide by zero (output values are
    validated non-zero before this call).
- **Fix direction**: use `checked_*`/`saturating_*` in `calc_tx_compute_mass` and
  `transaction_estimated_serialized_size`, or lower the mainnet `max_tx_*` limits to
  values that cannot overflow (as the params comment already recommends).

### 12. Signature hash (sighash) correctness — MEDIUM [NEW]

- **File**: `consensus/core/src/hashing/sighash.rs` (`calc_schnorr_signature_hash` :238,
  `calc_ecdsa_signature_hash` :267, `outputs_hash` :197, `payload_hash` :184),
  `sighash_type.rs`
- **What to verify**:
  - Sighash type validation (`SigHashType::from_u8`) rejects unknown bits; confirm
    `SIG_HASH_ALL` default and the `ANYONECANPAY`/`NONE`/`SINGLE` variants hash the
    correct subsets (a wrong subset = signature forgery across txs).
  - `outputs_hash` for `SINGLE` with out-of-range input index — confirm it returns the
    correct sentinel (upstream uses `0`/`ZERO_HASH`) and cannot panic.
  - Confirm the domain-separated hashers (`TransactionSigningHash`,
    `TransactionSigningHashECDSA`) are used consistently (Schnorr vs ECDSA).
- **Fix direction**: report any deviation; otherwise document as verified upstream.

### 13. Merkle root correctness — LOW [NEW]

- **File**: `crypto/merkle/src/lib.rs` (`calc_merkle_root` :3, `merkle_hash` :24)
- **What to verify**: empty input → `ZERO_HASH`; odd leaf count pads with `ZERO_HASH`;
  `next_power_of_two` and `2*next_pot - 1` sizing cannot overflow for realistic leaf
  counts; `merkles.last().unwrap().unwrap()` cannot panic (vec is non-empty when
  `len() > 0`). Confirm the branch hash uses `MerkleBranchHash` domain separation.
- **Fix direction**: document as verified; add a bound if leaf count is attacker-controlled.

### 14. Unsafe blocks inventory — MEDIUM [NEW]

- **Files**: `consensus/pow/src/matrix.rs:95-105` (`MaybeUninit` + `transmute`),
  `consensus/src/model/stores/contract.rs:38-46` (`ContractStorageKey::as_ref`
  `from_raw_parts` — already Phase 1 finding 17), any `unsafe` in `math/src/uint.rs`
  and `crypto/muhash/src/u3072.rs`
- **What to verify**: run `grep -rn "unsafe" consensus/ crypto/ math/` and audit each
  block for soundness (alignment, padding, initialization, provenance). Confirm the
  `#[repr(C)]` struct cast in `contract.rs` has no padding (32-byte array + u64 = 40
  bytes, no padding on 64-bit) and document the fragility.
- **Fix direction**: add `static_assertions`/`const` size checks or replace with safe
  serialization.

### 15. `debug_assert!` overflow checks in `math/src/uint.rs` operator impls — MEDIUM [NEW]

- **File**: `math/src/uint.rs:533-582` (`Add`/`Sub`/`Mul` use `debug_assert!(!carry)`)
- **What to verify**: in release builds, `Uint256`/`Uint3072` `+`/`-`/`*` silently wrap
  on overflow. Confirm every consensus use of these operators is provably within range
  (difficulty uses `Uint320`; muhash uses its own `mul` with explicit reduction; PoW
  target comparison uses `<=`). Flag any consensus path that relies on the
  `debug_assert!` for safety.
- **Fix direction**: document; for consensus-critical paths prefer explicit
  `overflowing_*`/`checked_*` handling.

### 16. Header validation ordering and block-acceptance bypass — HIGH [NEW]

- **File**: `consensus/src/pipeline/header_processor/pre_pow_validation.rs`
  (`check_difficulty_and_daa_score` :26, `check_pruning_violation` :14),
  `post_pow_validation.rs`, `pre_ghostdag_validation.rs`, `processor.rs`
- **What to verify**:
  - PoW is checked **after** cheap pre-PoW checks (parents, pruning, difficulty) and
    **before** GhostDAG/body processing — confirm no path accepts a block without PoW
    (e.g. `skip_proof_of_work` is only true for simnet).
  - `check_difficulty_and_daa_score` compares `header.bits != expected_bits` — confirm
    the expected bits come from the correct window (sampled vs legacy) and that a
    miner cannot manipulate the window to lower difficulty.
  - Timestamp/median-time checks (`past_median_time.rs`) — confirm the deviation
    tolerance and sampling activation match params.
- **Fix direction**: report any deviation; otherwise document as verified upstream.

---

## Execution steps for the builder

1. Create `audit_reports/` if missing (it already exists).
2. Work through the files in the priority order above. For each file:
   - Read the full file (use `offset`/`limit` for large files — `opcodes/mod.rs` is
     3886 lines, `tx.rs` is 819, `coinbase.rs` is 730).
   - Verify every finding's `file:line` against the current tree (line numbers above are
     from the audit snapshot and may drift — re-grep to confirm).
   - Reproduce the trigger mentally or with a minimal test where feasible.
   - Record the finding in the report format (severity, file:line, description, snippet, fix).
3. For the Zyanya-vs-upstream question: for each consensus-critical file, sanity-check
   against known upstream Kaspa/rusty-spectre behavior (the in-repo tests and comments
   are a good reference; the `SUBSIDY_BY_MONTH_TABLE` and `legacy_calc_block_subsidy`
   in `coinbase.rs` are the upstream reference implementations).
4. Add any additional findings discovered in the listed categories (integer overflow,
   consensus bypass, PoW/difficulty manipulation, hash collision resistance, Muhash
   correctness, address encoding, script VM safety, unsafe blocks, crypto correctness).
5. Sort the final report by severity (CRITICAL → HIGH → MEDIUM → LOW), then by file path.
6. Do **not** modify source code — this phase is audit + report only.

## Report skeleton

```markdown
# Phase 2 Findings — Consensus + Crypto

## CRITICAL
### [C-01] <title>
- **File**: <path>:<line>
- **Description**: ...
- **Code**:
  ```rust
  ...
  ```
- **Recommended fix**: ...

## HIGH
### [H-01] ...

## MEDIUM
### [M-01] ...

## LOW
### [L-01] ...
```

Use stable IDs (`C-01`, `H-01`, ...) so later phases can reference them. Reuse the
Phase 1 numbering convention but start fresh (Phase 2 IDs are independent).
