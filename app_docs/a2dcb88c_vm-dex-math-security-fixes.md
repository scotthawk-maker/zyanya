# Security Remediation Batch 1: VM & Smart Contract Math (F-H-01, F-H-02, F-H-03, F-H-07)

**Session**: a2dcb88c
**Scope**: Remediate four HIGH-severity findings from the final audit report covering VM arithmetic overflow, DEX constant-product math, jump-target validation, and contract payload size bounds.

---

## Summary

This change set closes four HIGH findings that all stem from unchecked or silently-wrapping arithmetic and insufficient input validation in the Zyanya VM and its smart-contract layer. The fixes replace wrapping operations with checked equivalents (returning explicit errors or reverting on overflow), validate jump targets against a pre-computed opcode-boundary map, and tighten contract payload size limits to prevent memory exhaustion during borsh deserialization.

| Finding | Severity | Root Cause | Fix |
|---------|----------|------------|-----|
| F-H-01 | HIGH | `POW` opcode used `wrapping_pow` and truncated the exponent to `u32` without validation | Validate exponent fits `u32`, use `checked_pow`, return `ArithmeticOverflow` on failure |
| F-H-02 | HIGH | DEX/bonding-curve arithmetic used unchecked `u64` multiplication that silently wraps on-chain; off-chain preview used `saturating_mul` on already-wrapped intermediates | On-chain ZCL contracts perform explicit overflow checks before each multiply, reverting (return 0) on overflow; off-chain explorer uses `u128` intermediates |
| F-H-03 | HIGH | Jump targets not found in the `byte_to_opcode` map were silently left as raw byte offsets, then compared against `code.len()` (opcode count) at runtime — allowing mid-opcode jumps | Deserialization now returns `InvalidJumpTarget` when a jump target does not resolve to a known opcode boundary |
| F-H-07 | HIGH | `MAX_CONTRACT_PARAMETERS` was derived from calldata size (`64 KB / 8 = 8192`), allowing excessive parameter counts | Reduced to 1024 per the audit recommendation; bytecode (1 MB), calldata (64 KB), and parameter count (1024) bounds are all enforced before borsh deserialization |

---

## Files Changed

### `zyanya-vm/src/vm.rs` — POW opcode handler (F-H-01)

The `OpCode::Pow` handler previously called `a.wrapping_pow(b as u32)`, silently wrapping on overflow and truncating the exponent from `u64` to `u32` without any validation.

**Changes:**
- Added a guard rejecting exponents larger than `u32::MAX` with `VMError::ArithmeticOverflow`.
- Gas is now computed on the validated `u64` exponent (`b / 32`) rather than the truncated `b as u64`, so the gas charge matches the actual exponent used.
- Replaced `wrapping_pow` with `checked_pow(b as u32).ok_or(VMError::ArithmeticOverflow)?`.

### `zyanya-vm/src/opcode.rs` — Jump-target validation (F-H-03)

During deserialization (`deserialize_slice`), jump targets are remapped from byte offsets to opcode indices via a `byte_to_opcode` HashMap. Previously, if a target was not found in the map, it was left as a raw byte offset and later compared against `code.len()` (opcode count) at runtime — a type confusion that could allow jumping into the middle of a multi-byte immediate.

**Changes:**
- Both `Jump` and `JumpIf` target resolution now returns `VMError::InvalidJumpTarget { pc, code_len }` when the target is not found in `byte_to_opcode`, instead of silently passing it through.

### `dex.zcl` — DEX constant-product swap (F-H-02)

The `swap` function computed `num = resB * amountIn * 997` and `den = resA * 1000 + amountIn * 997` with plain `u64` multiplication that silently wraps in the VM's `MUL` opcode (which uses `checked_mul` and returns overflow, but the ZCL source-level expressions were written as if overflow couldn't happen).

**Changes:**
- Added a zero-input guard (`if amountIn == 0 { return 0; }`).
- Each multiplication is preceded by an explicit overflow check using the `a > MAX / b` idiom; if any check fails, the function returns 0 (revert).
- A zero-denominator guard was added.
- Both swap directions (token A → B and token B → A) received the same treatment.

### `bonding_curve.zcl` — Bonding-curve buy/sell (F-H-02)

The `buy` and `sell` functions computed `cost = slope * (2 * supply * k + k * k) / 2` with unchecked arithmetic.

**Changes:**
- Each intermediate multiplication (`2 * supply`, `twoSupply * k`, `k * k`, `slope * inner`) is now preceded by an explicit overflow check.
- The `sell` path also guards against `term1 < term2` (underflow) even though the invariant `supply >= k` is expected to hold.
- On any overflow, the function returns 0 (revert).

### `zyanya-explorer/src/client.rs` — Off-chain preview arithmetic (F-H-02)

The explorer's buy/sell preview used `slope.saturating_mul(2 * S * k + k * k)` where the inner expression `2 * S * k + k * k` was plain `u64` arithmetic that could silently wrap before `saturating_mul` ever saw it.

**Changes:**
- Both buy and sell previews now cast operands to `u128`, compute `2 * S * k + k * k` (or subtraction for sell) in 128-bit space, then `saturating_mul` by `slope`, divide by 2, and clamp back to `u64` via `.min(u64::MAX as u128) as u64`.

### `consensus/core/src/config/constants.rs` — Parameter count limit (F-H-07)

`MAX_CONTRACT_PARAMETERS` was `MAX_CONTRACT_CALLDATA_SIZE / 8` (8192). The audit recommended capping it at 1024.

**Changes:**
- `MAX_CONTRACT_PARAMETERS` is now `1024`.
- The existing validation in `consensus/src/processes/transaction_validator/tx_validation_in_isolation.rs` already enforces `MAX_CONTRACT_BYTECODE_SIZE` (1 MB), `MAX_CONTRACT_CALLDATA_SIZE` (64 KB), `MAX_CONTRACT_PARAMETERS` (now 1024), and `MAX_CONTRACT_PAYLOAD_SIZE` before borsh deserialization — no changes were needed to that file beyond the constant update it consumes.

### Supporting files

- `audit_reports/SECURITY_AUDIT_PLAN.md` — New 254-line audit plan document created during this session, enumerating T0–T3 priority files and vulnerability categories for the independent security audit pass.
- `adws/adw_sssf_config/audit.config.yaml` — Added `audit_reports/` to the planner agent's write paths.
- `bonding_curve.zcl` — Also includes F-H-02 overflow-safe arithmetic (described above).

---

## Why It Matters

These findings are all in the financial-math and consensus-critical path:

- **F-H-01 (POW overflow)**: A contract could craft a large exponent to trigger silent wrapping, producing incorrect results that consensus nodes would all accept (since the wrap is deterministic), leading to wrong financial outcomes with no error signal.
- **F-H-02 (DEX/bonding-curve overflow)**: An attacker could supply crafted reserves/amounts to cause `u64` overflow in the constant-product formula, either griefing the DEX (revert on-chain) or obtaining incorrect off-chain quotes that mislead users. On-chain, the VM's `MUL` uses `checked_mul`, but the ZCL source expressions were written as unchecked chains — the fix ensures the ZCL code itself checks before multiplying so the contract reverts cleanly instead of relying on the VM's overflow trap alone.
- **F-H-03 (Jump-target validation)**: A malformed or malicious bytecode payload could encode a jump target pointing into the middle of a multi-byte opcode immediate. The old code would accept this as a valid opcode index (since it was just compared against `code.len()`), allowing arbitrary control-flow hijacking within a contract.
- **F-H-07 (Payload size bounds)**: Without explicit parameter-count limits, a malicious transaction could include an enormous `Vec<u64>` in the borsh payload, causing memory exhaustion during deserialization — a DoS vector against all consensus nodes.

---

## How to Verify

1. **Compilation**: `cargo check --workspace --all-targets` should pass with no errors.
2. **F-H-01**: Construct a VM test where `POW` is called with an exponent > `u32::MAX` — it should return `ArithmeticOverflow`. Also test `a.checked_pow(b)` overflow (e.g., `2.pow(64)`) — should also return `ArithmeticOverflow`.
3. **F-H-02**: In a DEX contract test, set reserves to large values near `u64::MAX` and attempt a swap — the contract should revert (return 0) rather than producing a wrapped (incorrect) `amountOut`. Repeat for the bonding curve. In the explorer, verify that a buy/sell preview with large supply/amount produces a clamped (not wrapped) result.
4. **F-H-03**: Construct bytecode with a `Jump`/`JumpIf` whose target byte offset does not align to any opcode start — deserialization should fail with `InvalidJumpTarget`.
5. **F-H-07**: Submit a contract invoke transaction with > 1024 parameters — it should be rejected at validation with a `MAX_CONTRACT_PARAMETERS` error. Verify bytecode > 1 MB and calldata > 64 KB are also rejected.
6. **Cross-reference**: The `audit_reports/SECURITY_AUDIT_PLAN.md` file documents the full audit scope and can be used to verify no other instances of these bug classes remain.