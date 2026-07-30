# Zyanya WASM SDK

An integration wrapper around `zyanya-wasm` module that uses
`websocket` W3C adaptor for WebSocket communication.

This is a Node.js module that provides bindings to the Zyanya WASM
SDK strictly for use in the Node.js environment. The web browser
version of the SDK is available as part of official SDK releases at
[https://github.com/zyanya-project/rusty-zyanya/releases](https://github.com/zyanya-project/rusty-zyanya/releases)

## Usage

Zyanya NPM module exports include all WASM32 bindings.
```javascript
const zyanya = require('zyanya');
console.log(zyanya.version());
```

## Documentation

As of now the code is compatible with Kaspa and its documentation can
be used from the official links.

## Building from source & Examples

SDK examples as well as information on building the project from
source can be found at [https://github.com/zyanya-project/rusty-zyanya/tree/main/wasm](https://github.com/zyanya-project/rusty-zyanya/tree/main/wasm)

## Releases

Official releases as well as releases for Web Browsers are available
at [https://github.com/zyanya-project/rusty-zyanya/releases](https://github.com/zyanya-project/rusty-zyanya/releases).
