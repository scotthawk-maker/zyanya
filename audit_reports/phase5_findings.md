# Phase 5 Security Audit — Explorer + Utils + Remaining Modules

**Date:** 2025-01-15  
**Scope:** Zyanya block explorer, database layer, daemon, mining, indexes, notify, math, utils, WASM, CLI, metrics, simpa, rothschild  
**Auditor:** Automated security review  
**Note:** The prompt's "Stratum protocol" focus area is **N/A** — there is no Stratum server in `mining/src/`. The mining module contains block-template building and mempool management only.

---

## CRITICAL

### C-01: Stored XSS via token metadata social links rendered into innerHTML

**File:** `zyanya-explorer/src/web.rs:3334-3339`  
**Source:** `zyanya-explorer/src/client.rs:1225-1234` (metadata saved from client-supplied `SignableTxData`)

**Description:**  
Token metadata fields (`twitter`, `telegram`, `website`) are rendered directly into `innerHTML` via template-literal interpolation without any HTML escaping or sanitization. An attacker who deploys a token with malicious metadata (e.g. `twitter: '"><img src=x onerror=alert(document.cookie)>'`) causes arbitrary JavaScript execution in the browser of any user visiting that token's page. The metadata is stored server-side in `token-metadata.json` and served to all visitors.

The flow:  
1. Attacker calls `/api/submit-signed-tx` with a `SignableTxData` payload containing `twitter: '"><img src=x onerror=alert(1)>'`  
2. Server saves it via `save_token_metadata()` at `client.rs:1234`  
3. Any user visiting `/token/<address>` triggers the `loadTokenPage()` JS which fetches `/api/token/<address>/metadata` and renders the social links into `innerHTML` at `web.rs:3339`

**Code snippet:**
```javascript
// web.rs:3334-3339
const socialsHtml = [];
if (meta.twitter) socialsHtml.push(`<a href="${meta.twitter}" target="_blank">Twitter / X</a>`);
if (meta.telegram) socialsHtml.push(`<a href="${meta.telegram}" target="_blank">Telegram</a>`);
if (meta.website) socialsHtml.push(`<a href="${meta.website}" target="_blank">Website</a>`);
document.getElementById('social-links').innerHTML = socialsHtml.join('');
```

```rust
// client.rs:1225-1234 — metadata saved from client-supplied SignableTxData
let metadata = TokenMetadata {
    name: Some(data.name.clone()),
    symbol: Some(data.symbol.clone()),
    description: data.description.clone(),
    twitter: data.twitter.clone(),    // ← attacker-controlled
    telegram: data.telegram.clone(),  // ← attacker-controlled
    website: data.website.clone(),    // ← attacker-controlled
    icon_uri: data.icon_uri.clone(),
};
self.save_token_metadata(&data.contract_address, metadata).await?;
```

**Recommended fix:**  
- Escape all metadata fields before rendering. Use `textContent` or create elements via `document.createElement('a')` and set `href` via `setAttribute`.  
- Alternatively, sanitize on the server side by stripping `<`, `>`, `"`, `'` characters from metadata fields before storage.  
- Validate that `twitter`/`telegram`/`website` are valid URLs (starting with `https://`).

---

### C-02: Stored XSS via token name/symbol rendered into innerHTML in token list

**File:** `zyanya-explorer/src/web.rs:1382-1383`  
**Source:** `zyanya-explorer/src/client.rs:1466-1468` (name/symbol from metadata store)

**Description:**  
The token listing page (`/explorer`) renders token `name` and `symbol` into an HTML string that is assigned to `innerHTML`. Since these values come from the metadata store (which is populated by client-supplied data via `submit_signed_tx`), an attacker can inject arbitrary HTML/JS that executes when any user views the token list.

**Code snippet:**
```javascript
// web.rs:1382-1383 — built into 'html' string, then set as innerHTML at line 1389
'<td class="mono" style="color:#7EC8D3; font-weight:bold;">' + t.name + ' (' + t.symbol + ')</td>' +
'<td class="mono">' + t.total_supply.toLocaleString() + ' ' + t.symbol + '</td>' +
```
```javascript
// web.rs:1389
document.getElementById('tokens-tbody').innerHTML = html;
```

**Recommended fix:**  
- Escape `t.name` and `t.symbol` before concatenation. Use a helper like `escapeHtml()` that replaces `<`, `>`, `&`, `"`, `'` with HTML entities.  
- Apply the same escaping to all other dynamic values rendered via `innerHTML` throughout `web.rs`.

---

### C-03: Unsigned transaction metadata injection — signatures do not cover metadata

**File:** `zyanya-explorer/src/client.rs:1180-1245`

**Description:**  
The `submit_signed_tx` function deserializes a `SignableTxData` struct from the client-supplied `unsigned_tx` hex payload. The Schnorr signatures only cover the transaction data (`data.tx` and `data.entries`), not the metadata fields (`name`, `symbol`, `description`, `twitter`, `telegram`, `website`, `icon_uri`, `contract_address`). An attacker can:

1. Request an unsigned deploy-token tx from the server with legitimate metadata
2. Modify the `SignableTxData` JSON to change metadata fields (e.g. inject XSS payloads into `twitter`)
3. Re-encode as hex and sign the unchanged sighashes
4. Submit the modified payload — signatures still validate because `data.tx` is unchanged
5. The server saves the attacker-controlled metadata

This enables both the stored XSS (C-01, C-02) and arbitrary metadata spoofing for any token.

**Code snippet:**
```rust
// client.rs:1182-1184 — deserialized from client hex, NOT from server's original response
let data: SignableTxData = serde_json::from_slice(&json_bytes)
    .map_err(|e| format!("Failed to parse unsigned transaction payload: {}", e))?;

// client.rs:1207-1209 — signatures verified against data.tx only
if let Err(e) = verify(&signable_tx.as_verifiable()) {
    return Err(format!("Signature verification failed: {:?}", e));
}

// client.rs:1225-1234 — metadata from the (tampered) data is saved directly
let metadata = TokenMetadata {
    name: Some(data.name.clone()),    // ← attacker can modify this
    symbol: Some(data.symbol.clone()), // ← attacker can modify this
    ...
};
self.save_token_metadata(&data.contract_address, metadata).await?;
```

**Recommended fix:**  
- Either sign the entire `SignableTxData` payload (not just the transaction), or  
- Store the metadata server-side when building the unsigned tx and look it up by tx-id after submission, ignoring client-supplied metadata fields.  
- At minimum, validate/sanitize all metadata fields before storage.

---

## HIGH

### H-01: Integer overflow in bonding curve buy calculation

**File:** `zyanya-explorer/src/client.rs:897`

**Description:**  
The bonding curve buy cost calculation uses non-saturating arithmetic for the intermediate expression `2 * S * k + k * k`, where `S` (total supply) and `k` (buy amount) are `u64` values read from on-chain contract state and user input respectively. While the outer multiplication uses `saturating_mul`, the inner arithmetic (`2 * S * k` and `k * k`) uses standard Rust multiplication which:
- **Panics** in debug mode (integer overflow)
- **Wraps silently** in release mode

In release mode, if `S` and `k` are large enough that `2 * S * k` overflows `u64`, the wrapping produces a small value, potentially allowing an attacker to buy tokens for near-zero cost.

**Code snippet:**
```rust
// client.rs:893-897
let S = total_supply;
let k = amount;
let cost = slope.saturating_mul(2 * S * k + k * k) / 2;
//                        ^^^^^^^^^^^^^^^^^^^^
//                        Non-saturating arithmetic — overflows in release!
```

**Recommended fix:**  
Use `u128` for intermediate calculations or use checked/saturating arithmetic throughout:
```rust
let cost = (slope as u128)
    .saturating_mul(2u128 * S as u128 * k as u128 + k as u128 * k as u128)
    / 2;
let cost = cost.min(u64::MAX as u128) as u64;
```

---

### H-02: Integer overflow in bonding curve sell (refund) calculation

**File:** `zyanya-explorer/src/client.rs:1045`

**Description:**  
Same issue as H-01 but in the sell/refund path. The expression `2 * S * k - k * k` uses non-saturating arithmetic. If `2 * S * k` overflows, the subtraction operates on a wrapped value, potentially producing an incorrect (possibly very large) refund amount.

**Code snippet:**
```rust
// client.rs:1041-1045
let S = total_supply;
let k = amount;
let refund = if S >= k {
    slope.saturating_mul(2 * S * k - k * k) / 2
    //                        ^^^^^^^^^^^^^^^^^^^^
    //                        Non-saturating arithmetic — overflows in release!
} else {
    0
};
```

**Recommended fix:**  
Same as H-01 — use `u128` intermediates or saturating/checked arithmetic.

---

### H-03: Reflected XSS via API error messages rendered into innerHTML

**File:** `zyanya-explorer/src/web.rs:3030, 3082, 3086, 3399, 3427, 3430`

**Description:**  
Multiple locations in the explorer's embedded JavaScript insert API error messages (which may contain user-supplied input) directly into `innerHTML` via template literals. The server constructs error strings that include user input (e.g. `format!("Invalid Zyanya address or public key format: {}", address_str)` at `client.rs:1568`), and the client renders these into `innerHTML` without escaping.

An attacker can craft input containing HTML/JS (e.g. address = `<img src=x onerror=alert(1)>`) which gets reflected through the error message into the victim's DOM.

**Code snippets:**
```javascript
// web.rs:3030
statusEl.innerHTML = `<span style="color: var(--burn-red)">Unsigned tx build error: ${unsignedData.error || 'Failed'}</span>`;

// web.rs:3082
statusEl.innerHTML = `<span style="color: var(--burn-red)">Submission Error: ${submitData.error || 'Submit failed'}</span>`;

// web.rs:3399
statusEl.innerHTML = `<span style="color: var(--burn-red)">Buy failed: ${data.error || 'Unknown error'}</span>`;

// web.rs:3427
statusEl.innerHTML = `<span style="color: var(--burn-red)">Sell failed: ${data.error || 'Unknown error'}</span>`;
```

**Recommended fix:**  
- Escape all error messages before inserting into `innerHTML`, or use `innerText`/`textContent` for error display.  
- Server-side: sanitize user input before including it in error strings.

---

### H-04: Ungated compile-contract endpoint allows CPU DoS on public explorers

**File:** `zyanya-explorer/src/api.rs:583-590`

**Description:**  
The `api_compile_contract_handler` endpoint does not call `check_write_enabled()`, unlike all other compute-intensive or state-changing endpoints (deploy, invoke, buy, sell, submit, transfer, swap). This means the endpoint is always available, even on public explorer deployments where `ZYANYA_EXPLORER_ENABLE_WRITE` is not set. An attacker can repeatedly submit large or complex contract source code for compilation, causing CPU exhaustion and potentially denying service to other explorer users.

**Code snippet:**
```rust
// api.rs:583-590 — no check_write_enabled() gate
pub async fn api_compile_contract_handler(
    State(client): State<Arc<RpcClientManager>>,
    Json(payload): Json<CompileContractReq>,
) -> Response {
    match client.compile_contract(&payload.source) {
        Ok(res) => Json(res).into_response(),
        Err(err) => (StatusCode::BAD_REQUEST, Json(serde_json::json!({ "error": err }))).into_response(),
    }
}
```

Compare with `api_deploy_contract_handler` at `api.rs:340` which properly gates:
```rust
if let Err(resp) = check_write_enabled() {
    return resp;
}
```

**Recommended fix:**  
Add `check_write_enabled()` gate to `api_compile_contract_handler`, or implement a separate rate-limiting/size-limiting mechanism for compile requests.

---

## MEDIUM

### M-01: Reflected XSS via block data in DAG visualization modal

**File:** `zyanya-explorer/src/web.rs:5712-5724`

**Description:**  
The DAG visualization page's block detail modal renders block data (hash, parents, selected_parent) from the API into `innerHTML` via template literals. While block hashes are typically hex strings, the `parents` array is joined with `<br>` tags and inserted raw. If any field contains unexpected content (e.g. via a malicious node feeding crafted data), it could lead to HTML injection.

**Code snippet:**
```javascript
// web.rs:5715-5721
body.innerHTML = `
    <div class="tt-row" ...><span ...>${block.hash || hash}</span></div>
    ...
    <div class="tt-row"><span class="tt-label">Parents:</span> <span class="tt-val" ...>${block.parents ? block.parents.join('<br>') : 'None'}</span></div>
    <div class="tt-row"><span class="tt-label">Selected Parent:</span> <span class="tt-val" ...>${block.selected_parent || 'None'}</span></div>
    ...
`;
```

**Recommended fix:**  
Escape all block data fields before interpolation into HTML template literals. Use `textContent` where possible.

---

### M-02: No CORS restrictions on explorer API

**File:** `zyanya-explorer/src/main.rs:67-101`

**Description:**  
The Axum router does not include any CORS middleware. When `ZYANYA_EXPLORER_ENABLE_WRITE=1` is set, any website can make cross-origin POST requests to state-changing endpoints (deploy, invoke, buy, sell, submit-signed-tx, transfer, swap). This enables cross-site request forgery (CSRF) attacks where a malicious website triggers token deployments, transfers, or trades on behalf of a user whose browser has access to the explorer.

**Code snippet:**
```rust
// main.rs:67 — no CORS layer, no middleware
let app = Router::new()
    .route("/api/deploy-contract", post(api_deploy_contract_handler))
    .route("/api/invoke-contract", post(api_invoke_contract_handler))
    .route("/api/submit-signed-tx", post(api_submit_signed_tx_handler))
    // ... all state-changing endpoints exposed without CORS
    .with_state(client_mgr);
```

**Recommended fix:**  
Add `tower-http::cors` middleware with restrictive origins, or implement CSRF token validation for state-changing endpoints. At minimum, set `Access-Control-Allow-Origin` to same-origin only.

---

### M-03: Unbounded notification channels can cause memory exhaustion DoS

**File:** `notify/src/subscriber.rs:75`, `notify/src/notifier.rs:293`

**Description:**  
Both the `Subscriber` and `Notifier` use `Channel::unbounded()` for their incoming notification channels. Under high notification volume (e.g. a burst of blocks or UTXO changes), notifications queue up without any backpressure. A sustained burst can cause unbounded memory growth, leading to OOM and process termination. There is no maximum queue depth or overflow policy.

**Code snippets:**
```rust
// subscriber.rs:75
Self {
    ...
    incoming: Channel::unbounded(),  // ← no bound
    ...
}

// notifier.rs:293
let notification_channel = Channel::unbounded();  // ← no bound
```

**Recommended fix:**  
Use bounded channels with a reasonable capacity (e.g. 10,000). When full, either drop oldest notifications or apply backpressure to the producer. Log warnings when the queue approaches capacity.

---

### M-04: Token icon upload has no content-type or size validation

**File:** `zyanya-explorer/src/client.rs:274-289`

**Description:**  
The `save_token_icon` function accepts arbitrary base64-encoded data, decodes it, and writes it to disk as a `.png` file. There is no validation that:
1. The decoded data is actually a valid PNG image
2. The decoded data size is within reasonable limits
3. The content does not contain malicious payloads (e.g. polyglot files)

An attacker could upload a large file (causing disk exhaustion) or a non-image file that gets served with `Content-Type: image/png` (causing browser confusion).

**Code snippet:**
```rust
// client.rs:274-289
pub fn save_token_icon(&self, address: &str, base64_data: &str) -> Result<String, String> {
    let decoded = decode_base64(base64_data)?;
    let filename = format!("{}.png", address);
    let mut path = std::path::Path::new(&self.icons_dir).join(&filename);
    // ... no PNG header validation, no size check
    std::fs::write(&path, &decoded)
}
```

**Recommended fix:**  
- Validate that decoded data starts with PNG magic bytes (`\x89PNG\r\n\x1a\n`)
- Enforce a maximum size (e.g. 256 KB)
- Consider re-encoding the image server-side to strip any embedded payloads

---

### M-05: Unsafe pointer cast in database registry relies on repr(u8) invariant

**File:** `database/src/registry.rs:87`

**Description:**  
The `AsRef<[u8]>` implementation for `DatabaseStorePrefixes` uses an unsafe pointer cast that transmutes `&Self` to `&u8` and then creates a slice from it. While the enum is `#[repr(u8)]` which makes this technically sound, the safety invariant is fragile — if the repr attribute is ever removed or changed, this becomes undefined behavior. The safety comment does not explain what would happen if the invariant is violated.

**Code snippet:**
```rust
// registry.rs:84-89
impl AsRef<[u8]> for DatabaseStorePrefixes {
    fn as_ref(&self) -> &[u8] {
        // SAFETY: enum has repr(u8)
        std::slice::from_ref(unsafe { &*(self as *const Self as *const u8) })
    }
}
```

**Recommended fix:**  
Replace with a safe alternative: `&[self as u8]` or use `[self as u8]` via the already-implemented `IntoIterator`. The unsafe block is unnecessary:
```rust
fn as_ref(&self) -> &[u8] {
    std::slice::from_ref(&(*self as u8))
}
```

---

### M-06: Fallback to world-readable /tmp directory for sensitive metadata

**File:** `zyanya-explorer/src/client.rs:227-230, 269, 283-288`

**Description:**  
When writing token metadata or icons fails (e.g. due to permissions), the code falls back to writing to `/tmp/zyanya-token-metadata.json` and `/tmp/zyanya-token-icons/`. The `/tmp` directory is typically world-readable on multi-user systems. This means:
1. Token metadata (which may contain social links, descriptions) is readable by any local user
2. The metadata file is overwritten on every save (no file locking), risking corruption
3. The fallback path is predictable, enabling symlink attacks

**Code snippet:**
```rust
// client.rs:267-270
if let Err(e) = std::fs::write(&self.metadata_path, &json) {
    let _ = std::fs::write("/tmp/zyanya-token-metadata.json", &json);
    log::warn!("Failed to write metadata to {}: {}, saved to /tmp", self.metadata_path, e);
}

// client.rs:283-288
if let Err(_) = std::fs::write(&path, &decoded) {
    let tmp_dir = std::path::Path::new("/tmp/zyanya-token-icons");
    let _ = std::fs::create_dir_all(tmp_dir);
    path = tmp_dir.join(&filename);
    std::fs::write(&path, &decoded)
}
```

**Recommended fix:**  
- Fail gracefully instead of falling back to `/tmp`
- If fallback is necessary, use a process-private directory (e.g. `$XDG_RUNTIME_DIR`) with restricted permissions (0700)
- Use atomic writes (write to temp file + rename) to prevent corruption

---

### M-07: Reflected XSS via error messages in DAG block modal

**File:** `zyanya-explorer/src/web.rs:5724`

**Description:**  
The block detail modal's error handler inserts `err.message` directly into `innerHTML`. If the fetch error message contains HTML (unlikely from fetch but possible from crafted responses), it could lead to HTML injection.

**Code snippet:**
```javascript
// web.rs:5724
body.innerHTML = `<div style="color: var(--side-red); padding: 1rem;">Failed to load block details: ${err.message}</div>`;
```

**Recommended fix:**  
Use `textContent` for error display, or escape `err.message` before insertion.

---

### M-08: Integer overflow in DAG handler pagination (limit + offset)

**File:** `zyanya-explorer/src/api.rs:181-183`

**Description:**  
The `api_dag_handler` computes `limit + offset` before passing it to `client.get_dag_graph()`. Both `limit` and `offset` are `usize` values parsed from query parameters. While `limit` is capped at 100 via `.min(100)`, `offset` has no upper bound. If a client sends `limit=100&offset=usize::MAX`, the addition `limit + offset` silently wraps around (in release mode) or panics (in debug mode). In release mode, the wrapped small value causes the handler to fetch a small DAG graph and then `.skip(offset)` with the original huge `offset`, producing an empty result — a denial-of-service vector. In debug mode, the panic crashes the request handler thread.

**Code snippet:**
```rust
// api.rs:181-183
let limit = pagination.limit.unwrap_or(20).min(100);
let offset = pagination.offset.unwrap_or(0);
match client.get_dag_graph(limit + offset).await {  // ← usize overflow
```

**Recommended fix:**  
Use checked addition and return a 400 Bad Request on overflow:
```rust
let total = limit.checked_add(offset).ok_or_else(|| /* return 400 */);
```
Or cap `offset` to a reasonable maximum (e.g. 10,000).

---

### M-09: Database connection builder panics on DB open failure

**File:** `database/src/db/conn_builder.rs:117, 126, 137`

**Description:**  
All three `build()` methods in `ConnBuilder` call `.unwrap()` on `DBWithThreadMode::open()`. If RocksDB fails to open the database (e.g. due to disk corruption, permissions, or a locked database), the process panics. While this is typically a startup-only operation, if the database path is attacker-controllable (e.g. via `--appdir` in a shared environment), a crafted path (very long, non-existent parent, etc.) could trigger the panic. Additionally, `self.db_path.to_str().unwrap()` can panic if the path contains non-UTF8 characters on non-Unix systems.

**Code snippet:**
```rust
// conn_builder.rs:117
let db = Arc::new(DB::new(
    <DBWithThreadMode<MultiThreaded>>::open(&opts, self.db_path.to_str().unwrap()).unwrap(),
    guard
));
```

**Recommended fix:**  
Replace `.unwrap()` with proper error propagation:
```rust
let db = DBWithThreadMode::open(&opts, self.db_path.to_str()
    .ok_or_else(|| /* error */)?)
    .map_err(|e| /* wrap error */)?;
```
The `delete_db` function at `db.rs:43` also uses `.expect("DB is expected to be deletable")` which will panic on failure — this should return a `Result` instead.

---

### M-10: Unbounded `ram_scale` f64 can cause resource exhaustion or division issues

**File:** `zyanyad/src/args.rs:80` (field), `mining/src/mempool/config.rs:apply_ram_scale`

**Description:**  
The `--ram-scale` CLI argument accepts any `f64` value with no bounds validation. The value is passed to `Config::apply_ram_scale()` which scales `maximum_transaction_count` by `ram_scale.min(1.0)`. While the `.min(1.0)` prevents scaling up the mempool, the value is also passed to `config.ram_scale = self.ram_scale` in `apply_to_config()` (args.rs) and used throughout consensus for memory allocation bounds. Special float values (NaN, Infinity, negative) could cause undefined behavior in downstream scaling calculations.

If `ram_scale` is set to `0.0`, `maximum_transaction_count` becomes 0, effectively disabling the mempool. If set to `NaN`, comparisons with `.min(1.0)` behave unexpectedly (NaN propagates), potentially causing panics or logic errors in memory allocation.

**Code snippet:**
```rust
// args.rs:80
pub ram_scale: f64,

// args.rs — apply_to_config
config.ram_scale = self.ram_scale;  // ← no validation

// config.rs — apply_ram_scale
self.maximum_transaction_count = (self.maximum_transaction_count as f64 * ram_scale.min(1.0)) as usize;
```

**Recommended fix:**  
Validate `ram_scale` at parse time:
```rust
if !ram_scale.is_finite() || ram_scale <= 0.0 || ram_scale > 100.0 {
    return Err("ram-scale must be a finite positive number between 0 and 100");
}
```

---

## LOW

### L-01: Unsafe impl Send for WASM Sink type

**File:** `wasm/core/src/events.rs:38`

**Description:**  
`Sink` contains `js_sys::Function` and `js_sys::Object` (JavaScript values), which are not truly `Send` in the Rust sense — they are tied to the JavaScript event loop. The `unsafe impl Send` allows these to be moved across threads in Rust's model, which is safe only in single-threaded WASM contexts. If this code is ever compiled for non-WASM targets or multi-threaded WASM (with shared memory), it could cause data races.

**Code snippet:**
```rust
// events.rs:38
unsafe impl Send for Sink {}
```

**Recommended fix:**  
- Gate the `Send` impl behind `#[cfg(target_arch = "wasm32")]`  
- Add a safety comment explaining the single-threaded WASM assumption  
- Consider using `Sendable` wrapper (already used elsewhere in the codebase) instead

---

### L-02: No rate limiting on explorer API endpoints

**File:** `zyanya-explorer/src/main.rs:67-101`

**Description:**  
None of the explorer's API endpoints implement rate limiting. A malicious client can flood the server with requests, causing excessive gRPC calls to the backend node and potentially exhausting its connection pool or causing degraded performance for legitimate users.

**Recommended fix:**  
Add rate-limiting middleware (e.g. `tower-governor`) with per-IP limits on resource-intensive endpoints (blocks, contracts, tokens, compile).

---

### L-03: Non-atomic metadata file writes risk corruption

**File:** `zyanya-explorer/src/client.rs:267`

**Description:**  
`std::fs::write` is not atomic — if the process crashes mid-write, the metadata file can be left in a truncated or corrupt state. Since the entire metadata store is serialized to a single JSON file on every token deployment, a crash during write would lose all token metadata.

**Code snippet:**
```rust
let json = serde_json::to_string_pretty(&*store)
    .map_err(|e| format!("Failed to serialize metadata: {}", e))?;
if let Err(e) = std::fs::write(&self.metadata_path, &json) {
```

**Recommended fix:**  
Use atomic write: write to a temporary file in the same directory, then rename to the target path. This ensures the file is either fully written or unchanged.

---

### L-04: UnboundedSender for transaction IDs in mining manager

**File:** `mining/src/manager.rs:39, 632, 932`

**Description:**  
The mining manager uses `tokio::sync::mpsc::UnboundedSender` for broadcasting transaction IDs to subscribers. If the receiver falls behind or disconnects, messages queue indefinitely. Under high transaction throughput, this can cause memory growth.

**Code snippet:**
```rust
// manager.rs:39
use tokio::sync::mpsc::UnboundedSender;

// manager.rs:632
pub fn revalidate_high_priority_transactions(
    &self,
    consensus: &dyn ConsensusApi,
    transaction_ids_sender: UnboundedSender<Vec<TransactionId>>,
)
```

**Recommended fix:**  
Use bounded channels, or handle `SendError` gracefully by logging and dropping stale messages. Consider using `try_send` with capacity checks.

---

### L-05: No input validation on CLI `--uacomment` values

**File:** `zyanyad/src/args.rs:228-233`

**Description:**  
The `--uacomment` CLI argument accepts arbitrary strings that are included in the node's user agent. While the values are typed as `String` (safe from injection in Rust), extremely long values or values containing control characters could cause issues in peer-to-peer handshake messages or log output.

**Code snippet:**
```rust
// args.rs:228-233
.arg(
    Arg::new("user_agent_comments")
        .long("uacomment")
        .action(ArgAction::Append)
        .require_equals(true)
        .help("Comment to add to the user agent -- See BIP 14 for more information."),
)
```

**Recommended fix:**  
Validate that user agent comments:
- Are under a reasonable length limit (e.g. 256 characters)
- Contain only printable ASCII characters
- Don't contain the `/` character (used as delimiter in user agent strings)

---

### L-06: Explorer does not set security headers

**File:** `zyanya-explorer/src/main.rs:67-101`

**Description:**  
The explorer's HTTP responses do not include standard security headers such as `X-Content-Type-Options: nosniff`, `X-Frame-Options: DENY`, `Content-Security-Policy`, or `Strict-Transport-Security`. This makes the XSS vulnerabilities described above more impactful and enables clickjacking attacks.

**Recommended fix:**  
Add `tower-http::set_header` middleware to set:
- `X-Content-Type-Options: nosniff`
- `X-Frame-Options: DENY`
- `Content-Security-Policy: default-src 'self'; script-src 'self'` (adjust as needed)
- `Strict-Transport-Security: max-age=31536000` (when TLS is used)

---

### L-07: Unsafe `from_utf8_unchecked` in WASM hex encoding

**File:** `wasm/core/src/types.rs:52`

**Description:**  
The `From<&[u8]>` implementation for `HexString` uses `unsafe { str::from_utf8_unchecked(&hex) }` after hex encoding. While `faster_hex::hex_encode` produces valid ASCII (and thus valid UTF-8), the unsafe call bypasses validation. If the `faster_hex` crate's behavior ever changes or the output buffer is corrupted, this could cause undefined behavior.

**Code snippet:**
```rust
// types.rs:48-53
fn from(bytes: &[u8]) -> Self {
    let mut hex = vec![0u8; bytes.len() * 2];
    faster_hex::hex_encode(bytes, hex.as_mut_slice()).expect("...");
    let result = unsafe { str::from_utf8_unchecked(&hex) };
    JsValue::from(result).into()
}
```

**Recommended fix:**  
Use the safe `str::from_utf8(&hex).expect("hex output is always valid UTF-8")` instead. The performance difference is negligible for hex-encoded data.

---

### L-08: DbKey Display indexes into path array without bounds checking

**File:** `database/src/key.rs:97-119`

**Description:**  
The `Display` implementation for `DbKey` indexes into `self.path` at positions `0` and `1` based on `self.prefix_len` checks. If `prefix_len` is `> 0` but `path` is empty (shouldn't happen in normal usage, but could via a malformed key from corrupted DB state), the index `self.path[0]` and `self.path[1]` would panic. Additionally, the `ReachabilityRelations` branch at line 108 indexes `self.path[1]` without checking if `self.path.len() > 1`. The `Display` trait is used in error messages and debug output, so corrupted DB keys could cause panics in logging code.

**Code snippet:**
```rust
// key.rs:99-108
if self.prefix_len > 0 {
    if let Ok(prefix) = DatabaseStorePrefixes::try_from(self.path[0]) {  // ← panics if path is empty
        ...
        if self.prefix_len > 1 {
            match prefix {
                ...
                ReachabilityRelations => {
                    if let Ok(next_prefix) = DatabaseStorePrefixes::try_from(self.path[1]) {  // ← panics if path.len() == 1
```

**Recommended fix:**  
Add bounds checks: `if self.prefix_len > 0 && !self.path.is_empty()` and `if self.prefix_len > 1 && self.path.len() > 1`. Alternatively, use `self.path.get(0)` and `self.path.get(1)` which return `Option`.

---

### L-09: `delete_db` panics on RocksDB destroy failure

**File:** `database/src/db.rs:42-43`

**Description:**  
The `delete_db` function calls `.expect("DB is expected to be deletable")` on the RocksDB `destroy` operation. If the database directory is locked by another process, has permission issues, or the path is invalid (non-UTF8), this causes a process panic. This function is called during `--reset-db` operations, so an environment issue (e.g., another node instance still running) would crash the node with an unhelpful panic message rather than a clean error.

**Code snippet:**
```rust
// db.rs:42-43
let path = db_dir.to_str().unwrap();
<DBWithThreadMode<MultiThreaded>>::destroy(&options, path).expect("DB is expected to be deletable");
```

**Recommended fix:**  
Return a `Result` and propagate the error to the caller, or at minimum use `.unwrap_or_else(|e| panic!("Failed to delete DB at {}: {}", path, e))` for a more informative panic.

---

### L-10: `network()` panics on conflicting network flags

**File:** `zyanyad/src/args.rs:170-180`

**Description:**  
The `network()` method uses a match expression that panics with `"only a single net should be activated"` if more than one of `mainnet`, `testnet`, `devnet`, `simnet` is set to `true`. This panic is reachable from user input — a user who passes both `--mainnet` and `--testnet` on the command line (or in a config file) will crash the node with an unhelpful panic message.

**Code snippet:**
```rust
// args.rs:170-180
pub fn network(&self) -> NetworkId {
    match (self.mainnet, self.testnet, self.devnet, self.simnet) {
        (true, false, false, false) => NetworkId::new(NetworkType::Mainnet),
        (false, true, false, false) => NetworkId::with_suffix(NetworkType::Testnet, self.testnet_suffix),
        (false, false, true, false) => NetworkId::new(NetworkType::Devnet),
        (false, false, false, true) => NetworkId::new(NetworkType::Simnet),
        (false, false, false, false) => NetworkId::new(NetworkType::Mainnet),
        _ => panic!("only a single net should be activated"),
    }
}
```

**Recommended fix:**  
Return a `Result<NetworkId, String>` and handle the error gracefully in the caller (e.g. print a helpful error message and exit with code 1).

---

### L-11: Daemon `.expect()` calls can crash on missing daemons

**File:** `daemon/src/lib.rs:99, 105`

**Description:**  
The `Daemons::zyanyad()` and `Daemons::cpu_miner()` methods call `.expect()` when the respective daemon is `None`. These are used in contexts where a user might not have configured the daemon, causing a panic with a message like "accessing Daemons::zyanyad while zyanyad option is None". This is a usability issue that manifests as a crash rather than a graceful error.

**Code snippet:**
```rust
// lib.rs:99
pub fn zyanyad(&self) -> Arc<dyn ZyanyadCtl + Send + Sync + 'static> {
    self.zyanyad.as_ref().expect("accessing Daemons::zyanyad while zyanyad option is None").clone()
}
```

**Recommended fix:**  
Return `Option<Arc<...>>` or `Result` and let the caller decide how to handle the missing daemon (e.g. show a user-friendly error message).

---

### L-12: Topological sort asserts no cycles with panic

**File:** `mining/src/model/topological_sort.rs:52`

**Description:**  
The `topological_sort` function ends with an `assert_eq!(sorted.len(), self.len(), "by definition, cryptographically no cycle can exist in a DAG of transactions")`. While the comment claims cycles cannot exist by cryptographic construction, a panic in this assertion would crash the mining process. The assertion is a debug invariant, but using `assert_eq!` means it also runs in release builds. If a bug or consensus failure somehow introduces a cycle, the node would crash instead of handling the error gracefully.

**Code snippet:**
```rust
// topological_sort.rs:52
assert_eq!(sorted.len(), self.len(), "by definition, cryptographically no cycle can exist in a DAG of transactions");
```

**Recommended fix:**  
Replace the assertion with a soft check that returns an error or logs a warning. Alternatively, use `debug_assert_eq!` to only check in debug builds, or return the partial sort with a warning.

---

### L-13: Orphan pool eviction is not truly random

**File:** `mining/src/mempool/model/orphan_pool.rs:210-212`

**Description:**  
The `get_random_low_priority_orphan` method uses `find()` which returns the **first** low-priority orphan, not a random one. This means that under orphan pool pressure, the same orphan is always evicted first (the one earliest in the map iteration order), creating a predictable eviction pattern. While this is not a direct security vulnerability, it could be exploited by an attacker who knows the eviction pattern to keep their malicious orphans in the pool while legitimate orphans are evicted.

**Code snippet:**
```rust
// orphan_pool.rs:210-212
fn get_random_low_priority_orphan(&self) -> Option<&MempoolTransaction> {
    self.all_orphans.values().find(|x| x.priority == Priority::Low)
}
```

**Recommended fix:**  
Use a proper random selection (e.g. `choose()` from the `rand` crate) or at least iterate from a random starting point. This matches the original Go implementation's intent.

---

### L-14: `check_transaction_standard_in_context` uses `assert!` for mass invariant

**File:** `mining/src/mempool/check_transaction_standard.rs:131`

**Description:**  
The `check_transaction_standard_in_context` function uses `assert!(contextual_mass > 0, "expected to be set by consensus")` to verify that the contextual mass has been set. This is a debug invariant, but using `assert!` means it also runs in release builds. If consensus fails to set the mass for a transaction (e.g. due to a bug or edge case), the node crashes instead of rejecting the transaction.

**Code snippet:**
```rust
// check_transaction_standard.rs:131
let contextual_mass = transaction.tx.mass();
assert!(contextual_mass > 0, "expected to be set by consensus");
```

**Recommended fix:**  
Replace with an error return: `if contextual_mass == 0 { return Err(NonStandardError::...); }` or use `debug_assert!`.

---

### L-15: Unsafe `from_utf8_unchecked` in math uint Display and LowerHex impls

**File:** `math/src/uint.rs:757, 816`

**Description:**  
The `Display` and `LowerHex` implementations for the `U192`/`U256` integer types use `unsafe { std::str::from_utf8_unchecked(...) }` to convert byte buffers to strings. The `Display` impl (line 816) operates on a buffer filled with decimal digit characters from `DEC_DIGITS_LUT`, and the `LowerHex` impl (line 757) operates on hex-encoded output from `faster_hex::hex_encode`. Both are indeed valid UTF-8/ASCII by construction, but the unsafe call bypasses validation. If the LUT or hex encoder ever produces invalid bytes (e.g. due to a bug or memory corruption), this would be undefined behavior.

**Code snippet:**
```rust
// uint.rs:757 — LowerHex impl
let str = unsafe { core::str::from_utf8_unchecked(&hex[first_non_zero..]) };

// uint.rs:816 — Display impl
let buf_str = unsafe { std::str::from_utf8_unchecked(&buf[curr..]) };
```

**Recommended fix:**  
Use safe `str::from_utf8(&buf).expect("buffer contains only ASCII digits/hex")` — the performance difference is negligible for formatting operations.

---

### L-16: Display LUT indexing could overflow on very large integer types

**File:** `math/src/uint.rs:783-804`

**Description:**  
The `Display` implementation uses a decimal digit lookup table (`DEC_DIGITS_LUT`) of size 200 bytes. The indices `d1 = (rem / 100) << 1` and `d2 = (rem % 100) << 1` are computed from `rem` which is the remainder of a `div_rem_u64(STEP)` call, so `rem < 10_000`. The indices `d1` and `d2` are at most `198 << 1 = 198` (since `rem < 10000`, `rem/100 < 100`, `d1 < 200`, `d1+1 < 200`), which fits within the 200-byte LUT. The indexing is therefore safe for the current types. However, the safety relies on the invariant that `rem < 10_000` which is guaranteed by `div_rem_u64` — if this function were ever to return an incorrect remainder, the LUT access would be out-of-bounds. This is an informational note: the code is correct as written but has implicit safety dependencies.

**Recommended fix:**  
Add a `debug_assert!(d1 < 200 && d2 < 200)` to catch any regression in `div_rem_u64`.

---

### L-17: `hex.rs` deserialize uses `unwrap()` on UTF-8 conversion

**File:** `utils/src/hex.rs:30`

**Description:**  
The `deserialize` function in the hex module calls `str::from_utf8(buff).unwrap()` on a byte slice obtained from `Deserialize::deserialize`. If the deserializer produces non-UTF8 bytes (which is possible for binary formats like borsh or postcard), this will panic. The unwrap is not guarded by error handling.

**Code snippet:**
```rust
// hex.rs:28-31
pub fn deserialize<'de, D, T>(deserializer: D) -> Result<T, D::Error>
where D: Deserializer<'de>, T: FromHex,
{
    let buff: &[u8] = Deserialize::deserialize(deserializer)?;
    T::from_hex(str::from_utf8(buff).unwrap()).map_err(D::Error::custom)  // ← unwrap
}
```

**Recommended fix:**  
Replace `unwrap()` with proper error handling: `str::from_utf8(buff).map_err(D::Error::custom)?`.

---

### L-18: `networking.rs` uses `unwrap()` on IP parsing in unroutable networks list

**File:** `utils/src/networking.rs:120`

**Description:**  
The `is_publicly_routable` method iterates over a hardcoded list of unroutable CIDR networks and calls `IpNet::from_str(curr_net).unwrap()` on each. While these are compile-time constants and the parsing should always succeed, if the constant list is ever modified with a typo, the unwrap would panic at runtime. This is a maintainability risk.

**Code snippet:**
```rust
// networking.rs:118-122
for curr_net in unroutable_nets {
    if IpNet::from_str(curr_net).unwrap().contains(&self.0) {  // ← unwrap on constant
        return false;
    }
}
```

**Recommended fix:**  
Parse the list once at compile time using `const` or `LazyLock`, or use `expect("unroutable_nets contains invalid CIDR")` for a more informative panic. Alternatively, use `if let Ok(net) = IpNet::from_str(curr_net)`.

---

### L-19: Rothschild logs private key in plaintext

**File:** `rothschild/src/main.rs:195-199`

**Description:**  
When a user provides a `--private-key` argument, rothschild logs the private key in plaintext via `info!`: `private key: {}` with `schnorr_key.display_secret()`. If logs are written to disk (which is the default for zyanyad), the private key is persisted in a log file that may be world-readable. Additionally, when no private key is provided, rothschild generates one and logs it with `sk.display_secret()`, encouraging the user to rerun with the key — but the key is already in the logs.

**Code snippet:**
```rust
// rothschild/src/main.rs:195-199
let mut log_message = format!(
    "Using Rothschild with:\n\tprivate key: {}\n\tfrom address: {}",
    schnorr_key.display_secret(),
    String::from(&zyanya_addr)
);
```

**Recommended fix:**  
- Never log private keys. Display them to stdout only (not through the logging framework).
- Ensure log files have restrictive permissions (0600).
- For the auto-generated key case, print to stdout directly rather than through `info!`.

---

### L-20: CLI wizard secrets handled via `Secret::new()` but trimmed with `as_bytes().to_vec()`

**File:** `cli/src/wizards/wallet.rs:68, 108, 112`

**Description:**  
The CLI wallet creation wizard collects wallet passwords and mnemonic passphrases via `term.ask(true, ...)` and wraps them in `Secret::new(... .trim().as_bytes().to_vec())`. While the `Secret` type provides zeroization on drop, the intermediate `String` returned by `term.ask()` is not zeroized — it lives in the Rust allocator until garbage collected. On systems with swap, the password string may be written to disk. Additionally, `trim()` creates a new `&str` slice but the original string still contains the full password with whitespace.

**Code snippet:**
```rust
// wallet.rs:68
let wallet_secret = Secret::new(term.ask(true, "Enter wallet encryption password: ").await?.trim().as_bytes().to_vec());
```

**Recommended fix:**  
- Use a zeroizing string type for the intermediate value returned by `term.ask()`.
- Clear the original `String` memory after extracting the trimmed bytes.
- Consider using `secrecy::SecretString` instead of `Secret<Vec<u8>>` for string-based secrets.

---

## Informational

### I-01: No Stratum server in mining module (N/A)

The prompt's focus area "Stratum protocol vulnerabilities" is not applicable. The `mining/src/` module contains block template building (`block_template/`), mempool management (`mempool/`), and mining manager orchestration (`manager.rs`). There is no Stratum protocol server implementation. Confirmed via grep: no matches for "stratum" or "Stratum" in `mining/src/`.

### I-02: Token icon handler has proper path traversal protection

The `token_icon_handler` at `api.rs:35-50` correctly uses `Path::file_name()` to strip path components before joining with the icons directory. This prevents path traversal attacks via the `:filename` URL parameter.

### I-03: Brand asset handler uses static match (no path traversal)

The `brand_asset_handler` at `api.rs:100-110` uses a match statement against known asset names, returning 404 for unknown assets. No path traversal risk.

### I-04: CLI link matchers use `open_external` with validated URLs

The CLI link matchers at `cli/src/matchers.rs` and `node-cli/src/matchers.rs` use regex patterns to extract URLs/hashes before opening them via `nw_sys::shell::open_external`. The regex patterns are restrictive (hex hashes, `http(s)://` URLs), limiting injection risk. However, the URL regex `http[s]?:\/\/\S+` is broad and could match crafted URLs — users should be cautious when clicking links in the terminal.

### I-05: Mining block template builder delegates to consensus — no direct coinbase/mass arithmetic

**Files reviewed:** `mining/src/block_template/builder.rs`, `mining/src/block_template/selector.rs`, `mining/src/block_template/model/tx.rs`

The `BlockTemplateBuilder::build_block_template` method delegates entirely to `consensus.build_block_template(miner_data.clone(), selector, build_mode)`. The mining crate does not perform coinbase amount calculations, mass arithmetic, or fee distribution — all of these are handled by the consensus layer. The `modify_block_template` method only replaces the coinbase payload and script public key, then recalculates the merkle root via `consensus.calc_transaction_hash_merkle_root`. No arithmetic vulnerabilities were found in the mining crate's block template code. The `selector.rs` file contains only trait/struct definitions with no computation.

### I-06: Math `overflowing_*` implementations reviewed — correct

**Files reviewed:** `math/src/uint.rs` (lines 67-193)

The `overflowing_shl`, `overflowing_shr`, `overflowing_add`, `overflowing_sub`, `overflowing_mul_u64`, and `overflowing_mul` implementations were reviewed. All correctly propagate carry/borrow/overflow using standard `u64::overflowing_*` primitives. The `overflowing_add` uses a `carrying_add_u64` helper that correctly chains carries. The `overflowing_mul` uses schoolbook multiplication with carry propagation. No correctness issues found.

### I-07: Utils `serde_bytes` deserializers reviewed — safe

**Files reviewed:** `utils/src/serde_bytes/de.rs`, `utils/src/serde_bytes_fixed/de.rs`, `utils/src/serde_bytes_fixed_ref/de.rs`

The serde_bytes deserializers use a `FromHexVisitor` pattern that properly handles `visit_str`, `visit_bytes`, `visit_borrowed_str`, etc. The `visit_bytes` path correctly calls `str::from_utf8(v).map_err(...)?` before hex decoding — no unsafe used. The fixed-size variants validate length before decoding. No vulnerabilities found.

### I-08: Utils `channel.rs`, `sync/rwlock.rs`, `sync/semaphore.rs`, `fd_budget.rs` reviewed — safe

**Files reviewed:** `utils/src/channel.rs`, `utils/src/sync/rwlock.rs`, `utils/src/sync/semaphore.rs`, `utils/src/fd_budget.rs`

- `channel.rs`: Wraps `async_channel` with a clean API. The `Default` impl uses `unbounded()` — callers should prefer `bounded()` when backpressure is needed (this is an API design choice, not a vulnerability).
- `sync/rwlock.rs`: Readers-first RW lock using a non-fair semaphore. Correct implementation, no data races.
- `sync/semaphore.rs`: Non-fair semaphore using atomics + event-listener. The `MAX_PERMITS = usize::MAX` is safe because `try_acquire` checks `count < permits` before decrementing. No overflow possible because `release` uses `fetch_add` and the counter starts at `MAX_PERMITS`.
- `fd_budget.rs`: Global FD counter with `compare_exchange` loop for atomic acquire. The `limit()` function uses `rlimit::getrlimit().unwrap()` on Unix — this could panic if the syscall fails, but this is a startup-only operation. The `Drop` impl uses `fetch_sub` which could underflow if a guard is double-dropped, but `FDGuard` is not `Clone` so this is not possible.

### I-09: Metrics `data.rs` counter overflow reviewed — handled

**File:** `metrics/core/src/data.rs`

The `per_sec` helper at line 714 uses `b.checked_sub(a).unwrap_or_default()` which correctly handles the case where `b < a` (counter reset/restart). The `MetricsData` struct uses `u64` counters which overflow at ~584 years of nanoseconds — not a practical concern. The `node_cpu_usage` calculation at line 683 (`b.node_cpu_usage as f64 / b.node_cpu_cores as f64 * 100.0`) could divide by zero if `node_cpu_cores` is 0, but this is a local metrics display issue, not a network-reachable vulnerability.

### I-10: Simpa reviewed — test/simulation tool, not production

**Files reviewed:** `simpa/src/main.rs`, `simpa/src/simulator/miner.rs`, `simpa/src/simulator/network.rs`

Simpa is a network simulation tool, not a production binary. It contains many `.unwrap()` calls, but these are in simulation/validation code where panics are acceptable (they indicate simulation errors). No security-relevant findings. The `ram_scale` argument is also unbounded here, consistent with the M-10 finding, but simpa is not exposed to network attackers.

### I-11: Rothschild reviewed — test/load tool, not production

**File:** `rothschild/src/main.rs`

Rothschild is a transaction load generator. It uses `.unwrap()` extensively on RPC responses, which would crash the tool on network errors — acceptable for a testing tool. The L-19 finding (private key logging) is the main security concern. No other security-relevant findings.

### I-12: Integration tests reviewed — test-only code

**Files reviewed:** `testing/integration/src/` (overview scan)

Integration tests use `.unwrap()` and `.expect()` extensively, which is standard for test code. No security vulnerabilities — these are not compiled into production binaries.

### I-13: RBF implementation reviewed — correct

**File:** `mining/src/mempool/replace_by_fee.rs`

The RBF implementation correctly handles all three policies (`Forbidden`, `Allowed`, `Mandatory`). The `Allowed` policy validates all double-spends before removing any, preventing partial-replacement inconsistency. The feerate comparison uses `f64` which could have precision issues, but the comparison is strict (`>`) and the values are derived from integer fee/mass ratios, so precision loss is unlikely to cause a security issue. No vulnerabilities found.

### I-14: Orphan pool size limiting reviewed — bounded

**File:** `mining/src/mempool/model/orphan_pool.rs`

The orphan pool enforces `maximum_orphan_transaction_count` (default 500) via `limit_orphan_pool_size()`. It also checks `maximum_orphan_transaction_mass` (default 100,000) per orphan. Transactions are expired based on `orphan_expire_interval_daa_score`. The pool growth is bounded by configuration. The L-13 finding (non-random eviction) is a minor weakness, not a vulnerability.

---

## Summary

| Severity | Count | IDs |
|----------|-------|-----|
| CRITICAL | 3 | C-01, C-02, C-03 |
| HIGH | 4 | H-01, H-02, H-03, H-04 |
| MEDIUM | 10 | M-01 through M-10 |
| LOW | 20 | L-01 through L-20 |
| Info | 14 | I-01 through I-14 |
| **Total** | **51** | |

The most severe findings are the stored XSS chain (C-01 + C-02 + C-03) which allows an attacker to execute arbitrary JavaScript in any visitor's browser by deploying a token with malicious metadata. The metadata is not covered by the Schnorr signature, so it can be freely tampered. The integer overflow issues (H-01, H-02) in the bonding curve calculations could allow economic exploitation (buying/selling tokens at incorrect prices) in release-mode binaries.

New findings in this revision:
- **M-08**: DAG handler `limit + offset` usize overflow
- **M-09**: Database `ConnBuilder::build()` panics on DB open failure
- **M-10**: Unbounded `ram_scale` f64 can cause resource exhaustion
- **L-08**: `DbKey` Display indexes into path array without bounds checking
- **L-09**: `delete_db` panics on RocksDB destroy failure
- **L-10**: `network()` panics on conflicting network flags
- **L-11**: Daemon `.expect()` calls crash on missing daemons
- **L-12**: Topological sort asserts no cycles with panic in release builds
- **L-13**: Orphan pool eviction is not truly random
- **L-14**: `check_transaction_standard_in_context` uses `assert!` for mass invariant
- **L-15**: Unsafe `from_utf8_unchecked` in math uint Display and LowerHex impls
- **L-16**: Display LUT indexing safety depends on `div_rem_u64` invariant
- **L-17**: `hex.rs` deserialize uses `unwrap()` on UTF-8 conversion
- **L-18**: `networking.rs` uses `unwrap()` on IP parsing in unroutable networks list
- **L-19**: Rothschild logs private key in plaintext
- **L-20**: CLI wizard secrets not fully zeroized

New informational notes covering previously missing areas:
- **I-05**: Mining block template builder (no direct arithmetic — delegated to consensus)
- **I-06**: Math `overflowing_*` implementations (correct)
- **I-07**: Utils `serde_bytes` deserializers (safe)
- **I-08**: Utils `channel.rs`, `sync/`, `fd_budget.rs` (safe)
- **I-09**: Metrics counter overflow (handled)
- **I-10**: Simpa (test tool, not production)
- **I-11**: Rothschild (test tool, L-19 applies)
- **I-12**: Integration tests (test-only code)
- **I-13**: RBF implementation (correct)
- **I-14**: Orphan pool size limiting (bounded)