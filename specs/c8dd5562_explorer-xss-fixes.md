# Remediation Plan — Group E: Explorer XSS Fixes (F-C-14, F-C-15, F-C-16)

**Date:** 2026-08-18
**Scope:** `zyanya-explorer` crate only. Fix three CRITICAL findings from `audit_reports/FINAL_AUDIT_REPORT.md` and add a general HTML-escape utility applied to all user-controlled content rendered by the explorer.

**Verification gate:** `source /root/.cargo/env && cargo check -p zyanya-explorer 2>&1` must pass. Do **not** run full build or tests.

---

## 1. Findings summary

| ID | Severity | Root cause | File (current line refs) |
|----|----------|-----------|--------------------------|
| F-C-16 | CRITICAL | Signatures cover only `data.tx`/`data.entries`; metadata fields (`name`, `symbol`, `description`, `twitter`, `telegram`, `website`, `icon_uri`) are client-supplied and saved verbatim. | `zyanya-explorer/src/client.rs:1180-1245` (save at `:1225-1234`) |
| F-C-14 | CRITICAL | Token metadata social links (`twitter`, `telegram`, `website`) interpolated into `innerHTML` without escaping. | `zyanya-explorer/src/web.rs:3334-3339` (in `TOKEN_HTML`) |
| F-C-15 | CRITICAL | Token `name`/`symbol` interpolated into `innerHTML` in the token list without escaping. | `zyanya-explorer/src/web.rs:1382-1383` (in `EXPLORER_HTML`) |

F-C-16 is the **root cause** that enables F-C-14/F-C-15 (attacker-controlled metadata reaches the DOM). Fix it first (server-side), then harden the DOM rendering (client-side), then apply the escape utility broadly.

---

## 2. Files to modify

| File | Change |
|------|--------|
| `zyanya-explorer/src/client.rs` | Add `sanitize_metadata_field` + `sanitize_metadata` helpers; apply to all metadata before `save_token_metadata` in `submit_signed_tx`; apply again inside `save_token_metadata` (defense-in-depth). |
| `zyanya-explorer/src/web.rs` | Add `escapeHtml()` + `safeHttpsUrl()` JS helpers; fix `loadTokens` (F-C-15), `loadTokenData` social links (F-C-14); apply escaping to other user-controlled `innerHTML` sinks. |

No other crates are touched. No dependency changes.

---

## 3. Priority order

1. **F-C-16 (server-side sanitization)** — `client.rs`. Stops malicious metadata from ever being persisted/served. Highest priority because it is the source of the stored XSS payloads.
2. **F-C-14 (social links)** — `web.rs` `TOKEN_HTML`. Defense-in-depth for the token detail page.
3. **F-C-15 (token list name/symbol)** — `web.rs` `EXPLORER_HTML`. Defense-in-depth for the token list page.
4. **General `escapeHtml` rollout** — apply to remaining user-controlled `innerHTML` sinks in `web.rs`.
5. **`cargo check`** — fix any compilation errors.

---

## 4. Detailed fix steps

### 4.1 F-C-16 — Server-side metadata sanitization (`zyanya-explorer/src/client.rs`)

**Location:** `submit_signed_tx` builds `TokenMetadata` at `client.rs:1225-1234` and calls `self.save_token_metadata(...)`.

**Approach:** Implement the "at minimum" fix from the report — sanitize all metadata fields on save by stripping `<`, `>`, `"`, `'` (and backtick + control chars for completeness). Also validate URL fields (`twitter`, `telegram`, `website`) so non-empty values must start with `https://`; otherwise blank them. This is simpler and safer than re-signing metadata and does not break the existing Schnorr signing flow.

**Steps:**

1. Add a module-level helper near the top of `client.rs` (after the struct definitions, before `impl RpcClientManager`):

```rust
/// Strips characters that are dangerous in HTML/attribute contexts.
/// Removes `<`, `>`, `"`, `'`, backtick, and C0 control chars.
fn sanitize_metadata_field(value: Option<String>) -> Option<String> {
    let v = value?;
    let cleaned: String = v
        .chars()
        .filter(|c| !matches!(c, '<' | '>' | '"' | '\'' | '`') && !c.is_control())
        .collect();
    let trimmed = cleaned.trim().to_string();
    if trimmed.is_empty() { None } else { Some(trimmed) }
}

/// URL fields must be plain https:// links; anything else is dropped.
fn sanitize_metadata_url(value: Option<String>) -> Option<String> {
    let v = sanitize_metadata_field(value)?;
    if v.starts_with("https://") { Some(v) } else { None }
}

fn sanitize_metadata(m: TokenMetadata) -> TokenMetadata {
    TokenMetadata {
        name: sanitize_metadata_field(m.name),
        symbol: sanitize_metadata_field(m.symbol),
        description: sanitize_metadata_field(m.description),
        twitter: sanitize_metadata_url(m.twitter),
        telegram: sanitize_metadata_url(m.telegram),
        website: sanitize_metadata_url(m.website),
        icon_uri: sanitize_metadata_field(m.icon_uri),
    }
}
```

2. In `submit_signed_tx`, wrap the metadata construction (`client.rs:1225-1234`) with `sanitize_metadata(...)`:

```rust
let metadata = sanitize_metadata(TokenMetadata {
    name: Some(data.name.clone()),
    symbol: Some(data.symbol.clone()),
    description: data.description.clone(),
    twitter: data.twitter.clone(),
    telegram: data.telegram.clone(),
    website: data.website.clone(),
    icon_uri: data.icon_uri.clone(),
});
self.save_token_metadata(&data.contract_address, metadata).await?;
```

3. Defense-in-depth: also sanitize inside `save_token_metadata` (`client.rs:259`) so **any** future caller is covered:

```rust
pub async fn save_token_metadata(&self, address: &str, metadata: TokenMetadata) -> Result<(), String> {
    let metadata = sanitize_metadata(metadata);
    // ... existing body unchanged ...
}
```

4. Confirm `TokenMetadata` derives `Clone` (it does — `#[derive(Debug, Clone, Serialize, Deserialize, Default)]`), so `sanitize_metadata` can take it by value and return a new one.

**Note on the stronger fix (optional, not required):** The report's preferred fix is to sign the full `SignableTxData` payload. That requires changing the sighash computation in `build_unsigned_deploy_token_tx` and the verification in `submit_signed_tx`, and would break compatibility with already-issued unsigned payloads. The sanitization approach is the accepted minimum and is what this plan implements. Do **not** attempt the signature change in this task.

---

### 4.2 F-C-14 — Social links rendered into `innerHTML` (`zyanya-explorer/src/web.rs`, `TOKEN_HTML`)

**Location:** `loadTokenData()` at `web.rs:3334-3339`:

```javascript
const socialsHtml = [];
if (meta.twitter) socialsHtml.push(`<a href="${meta.twitter}" target="_blank">Twitter / X</a>`);
if (meta.telegram) socialsHtml.push(`<a href="${meta.telegram}" target="_blank">Telegram</a>`);
if (meta.website) socialsHtml.push(`<a href="${meta.website}" target="_blank">Website</a>`);
document.getElementById('social-links').innerHTML = socialsHtml.join('');
```

**Fix:** Replace with DOM-element construction using `createElement` + `setAttribute`, and only render links whose URL passes `safeHttpsUrl`:

```javascript
const socialsContainer = document.getElementById('social-links');
socialsContainer.innerHTML = '';
const socials = [
    { key: 'twitter', label: 'Twitter / X' },
    { key: 'telegram', label: 'Telegram' },
    { key: 'website', label: 'Website' },
];
for (const s of socials) {
    const url = safeHttpsUrl(meta[s.key]);
    if (!url) continue;
    const a = document.createElement('a');
    a.setAttribute('href', url);
    a.setAttribute('target', '_blank');
    a.setAttribute('rel', 'noopener noreferrer');
    a.textContent = s.label;
    socialsContainer.appendChild(a);
}
```

This removes the `innerHTML` sink entirely for social links and enforces `https://` at render time.

---

### 4.3 F-C-15 — Token name/symbol in token list (`zyanya-explorer/src/web.rs`, `EXPLORER_HTML`)

**Location:** `loadTokens()` at `web.rs:1382-1383` (built into `html`, assigned to `innerHTML` at `:1389`):

```javascript
'<td class="mono" style="color:#7EC8D3; font-weight:bold;">' + t.name + ' (' + t.symbol + ')</td>' +
'<td class="mono">' + t.total_supply.toLocaleString() + ' ' + t.symbol + '</td>' +
```

**Fix:** Escape `t.name` and `t.symbol` (and the address-derived `shortAddr` / `t.contract_address` used in the `onclick` attribute) with `escapeHtml`:

```javascript
const shortAddr = escapeHtml(t.contract_address.substring(0, 12) + '...' + t.contract_address.substring(t.contract_address.length - 8));
const addr = escapeHtml(t.contract_address);
const name = escapeHtml(t.name);
const symbol = escapeHtml(t.symbol);
html += '<tr>' +
    '<td><a href="#" class="link mono" onclick="viewContract(\'' + addr + '\')">' + shortAddr + '</a></td>' +
    '<td class="mono" style="color:#7EC8D3; font-weight:bold;">' + name + ' (' + symbol + ')</td>' +
    '<td class="mono">' + t.total_supply.toLocaleString() + ' ' + symbol + '</td>' +
    '<td class="mono">' + t.bytecode_size.toLocaleString() + ' bytes</td>' +
    '<td><a href="/tools" class="nav-btn mono" style="padding:0.25rem 0.6rem; font-size:0.75rem; text-decoration:none;">TRANSFER</a></td>' +
'</tr>';
```

---

### 4.4 General `escapeHtml` utility + rollout

**Add these two JS helpers** to the `<script>` block of each HTML constant that renders user-controlled content. The three pages that need them are:

- `EXPLORER_HTML` (script starts ~`web.rs:1220`) — token list, contract list, block detail, contract state query.
- `TOKEN_HTML` (script starts ~`web.rs:3310`) — token detail page.
- `LAUNCH_HTML` (script starts ~`web.rs:2950`) — launch confirmation modal echoes `name`/`symbol`/`user_address`.

Insert near the top of each script block (before first use):

```javascript
function escapeHtml(str) {
    if (str === null || str === undefined) return '';
    return String(str)
        .replace(/&/g, '&amp;')
        .replace(/</g, '&lt;')
        .replace(/>/g, '&gt;')
        .replace(/"/g, '&quot;')
        .replace(/'/g, '&#39;');
}

function safeHttpsUrl(url) {
    if (typeof url !== 'string') return null;
    const trimmed = url.trim();
    if (!/^https:\/\//i.test(trimmed)) return null;
    return trimmed;
}
```

**Apply `escapeHtml` to the remaining user-controlled `innerHTML` sinks** (defense-in-depth; these are the ones reachable with attacker-influenced strings):

| Line (approx) | Sink | User-controlled value(s) | Action |
|---------------|------|--------------------------|--------|
| `web.rs:1364` | `contract-query-result` | `key` (search input), `data.value` (contract storage) | wrap `key` and `data.value` in `escapeHtml` |
| `web.rs:1440-1446` | `detail-body` (contract view) | `address` (search input), `code.deploy_tx_id`, `code.bytecode_hex` | wrap `address`, `code.deploy_tx_id`, `code.bytecode_hex` in `escapeHtml` |
| `web.rs:1341-1346` | `loadContracts` | `c.address`, `c.contract_type` | wrap `c.address` (and `shortAddr`) in `escapeHtml` |
| `web.rs:1405-1415` | `loadDex` | `addr`/`d.address` | wrap `addr` in `escapeHtml` |
| `web.rs:1300-1315` | `viewBlock` `detail-body` | `o.address` (coinbase output address) | wrap `o.address` in `escapeHtml` |
| `web.rs:3030-3086` | `statusEl` (launch page) | `${unsignedData.error}`, `${submitData.error}`, `${err.message}` (reflected server/error text) | wrap these interpolations in `escapeHtml` |
| `web.rs:3035-3045` | `modalSummaryContent` (launch page) | `${summary.name}`, `${summary.symbol}`, `${summary.user_address}` | wrap in `escapeHtml` |
| `web.rs:3396-3430` | `statusEl` (token page buy/sell) | `${data.error}`, `${err.message}` | wrap in `escapeHtml` |

**Do NOT change** the static/trusted `innerHTML` sinks (brand SVG injection at `web.rs:659`, `1223`, `2522`, `2610`, `3955`, `4466`, `5318`; the DAG tooltip at `5683` which renders node hashes; block-detail `body.innerHTML` at `5704-5724` which renders node-supplied hex hashes). These are out of scope for this task and changing them risks breaking SVG rendering. If time permits, the DAG tooltip hash fields may be escaped, but it is not required.

---

## 5. Vulnerability categories per file

### `zyanya-explorer/src/client.rs`
- **Input validation / injection (CRITICAL):** unsigned-tx metadata fields are trusted and persisted without sanitization (F-C-16).
- **Defense-in-depth:** sanitize at the persistence boundary (`save_token_metadata`) so no future caller can bypass.

### `zyanya-explorer/src/web.rs`
- **Stored XSS via `innerHTML` (CRITICAL):** token metadata social links (F-C-14) and token name/symbol (F-C-15).
- **Reflected XSS via `innerHTML` (HIGH, related F-H-26):** API error messages and search-input values echoed into `innerHTML`.
- **URL scheme validation (MEDIUM):** `javascript:`/`data:` URLs in `href` attributes.

---

## 6. Verification

1. `source /root/.cargo/env && cargo check -p zyanya-explorer 2>&1`
2. Fix any compilation errors introduced by the changes (e.g. borrow/move issues in `sanitize_metadata`, JS string escaping inside the Rust raw strings).
3. Confirm no other crate is affected (only `zyanya-explorer` is checked).
4. Do **not** run `cargo build`, `cargo test`, or any full build.

**Watch-outs for the builder:**
- The JS lives inside Rust raw strings (`r###"..."###` and `r#"..."#`). The new JS must not contain the raw-string terminator sequence (`"###` / `"#`). The proposed helpers use only single quotes and backticks — safe. Avoid adding `"###` or `"#` inside the JS.
- `escapeHtml` uses `String(str)` and `.replace(...)` — plain ES5/ES6, no template literals needed inside the helper itself.
- `sanitize_metadata` takes `TokenMetadata` by value; `TokenMetadata` already derives `Clone`, so no extra derives needed.
- Keep the `https://` check case-insensitive on the JS side (`/^https:\/\//i`) and exact-prefix on the Rust side (`starts_with("https://")`) — document the slight asymmetry is acceptable because the Rust side is the authoritative gate.
