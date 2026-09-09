#!/usr/bin/env bash
# ==============================================================================
# Zyanya Cloud Seed Relay Node One-Click Deployment Script
# Targets: London (EU) & Tokyo (APAC) Cloud VPS Instances
# ==============================================================================

set -euo pipefail

REGION="${1:-london}"
NETWORK="${2:-mainnet}"

echo "======================================================================"
echo "🚀 ZYANYA CLOUD SEED RELAY DEPLOYMENT: [${REGION^^}] (${NETWORK^^})"
echo "======================================================================"

# 1. Update packages and install dependencies
echo "Step 1/5: Installing system packages and firewall..."
apt-get update -qq
apt-get install -y -qq ufw curl jq git ca-certificates

# 2. Configure UFW Firewall rules for Zyanya
echo "Step 2/5: Configuring UFW security rules..."
if [ "$NETWORK" = "mainnet" ]; then
    P2P_PORT="18111"
    RPC_PORT="18110"
    WRPC_PORT="19110"
else
    P2P_PORT="18211"
    RPC_PORT="18210"
    WRPC_PORT="19210"
fi

ufw allow 22/tcp comment "SSH"
ufw allow ${P2P_PORT}/tcp comment "Zyanya P2P Mesh"
ufw allow ${WRPC_PORT}/tcp comment "Zyanya wRPC"
ufw --force enable

# 3. Create system user & working directories
echo "Step 3/5: Setting up system user and storage..."
id -u zyanya &>/dev/null || useradd -m -s /bin/bash zyanya
mkdir -p /home/zyanya/.zyanyad /opt/zyanya/bin
chown -R zyanya:zyanya /home/zyanya /opt/zyanya

# 4. Download latest compiled binary release
echo "Step 4/5: Installing zyanyad binary..."
if [ ! -f /opt/zyanya/bin/zyanyad ]; then
    # When hosted on GitHub releases:
    # curl -fsSL "https://github.com/scotthawk-maker/zyanya/releases/latest/download/zyanyad-linux-x86_64" -o /opt/zyanya/bin/zyanyad
    echo "⚠️ Please place precompiled zyanyad into /opt/zyanya/bin/zyanyad"
fi
chmod +x /opt/zyanya/bin/zyanyad || true
ln -sf /opt/zyanya/bin/zyanyad /usr/local/bin/zyanyad || true

# 5. Create systemd Service Unit
echo "Step 5/5: Installing systemd daemon service..."
PEER_FLAGS=""
if [ "$REGION" = "london" ]; then
    PEER_FLAGS="--addpeer=us-seed.zyanya.org:${P2P_PORT}"
elif [ "$REGION" = "tokyo" ]; then
    PEER_FLAGS="--addpeer=us-seed.zyanya.org:${P2P_PORT} --addpeer=eu-seed.zyanya.org:${P2P_PORT}"
fi

cat <<EOF > /etc/systemd/system/zyanyad.service
[Unit]
Description=Zyanya GhostDAG Node Daemon (${REGION})
After=network-online.target
Wants=network-online.target

[Service]
Type=simple
User=zyanya
Group=zyanya
WorkingDirectory=/home/zyanya
ExecStart=/usr/local/bin/zyanyad --${NETWORK} --listen=0.0.0.0:${P2P_PORT} --rpclisten-borsh=0.0.0.0:${WRPC_PORT} --utxoindex ${PEER_FLAGS} --perf-metrics
Restart=always
RestartSec=5s
LimitNOFILE=65536

[Install]
WantedBy=multi-user.target
EOF

systemctl daemon-reload
echo "======================================================================"
echo "✅ ZYANYA ${REGION^^} SEED NODE SETUP COMPLETE!"
echo "To start the node: systemctl start zyanyad"
echo "To check live logs: journalctl -u zyanyad -f"
echo "======================================================================"
