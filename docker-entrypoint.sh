#!/usr/bin/env bash
set -e

NETWORK="${ZYANYA_NETWORK:-mainnet}"

# If first arg is a flag, default to zyanyad
if [ "${1:0:1}" = '-' ]; then
    set -- zyanyad "$@"
fi

if [ "$1" = "zyanyad" ]; then
    shift
    if [ "$NETWORK" = "testnet" ] || [ "$NETWORK" = "testnet-10" ]; then
        echo "=== [Zyanya] Starting Node on TESTNET-10 (gRPC: 18210, P2P: 18211) ==="
        exec /usr/local/bin/zyanyad \
            --testnet \
            --listen=0.0.0.0:18211 \
            --rpclisten=0.0.0.0:18210 \
            --rpclisten-borsh=0.0.0.0:19210 \
            --rpclisten-json=0.0.0.0:20210 \
            --utxoindex \
            "$@"
    else
        echo "=== [Zyanya] Starting Node on MAINNET (gRPC: 18110, P2P: 18111) ==="
        exec /usr/local/bin/zyanyad \
            --listen=0.0.0.0:18111 \
            --rpclisten=0.0.0.0:18110 \
            --rpclisten-borsh=0.0.0.0:19110 \
            --rpclisten-json=0.0.0.0:20110 \
            --utxoindex \
            "$@"
    fi
fi

exec "$@"
