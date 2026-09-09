#!/usr/bin/env bash
# ==============================================================================
# Zyanya GhostDAG L1 - Headless One-Liner Installer & Node Provisioner
# Usage: curl -sSfL https://zyanya.org/install.sh | bash
# Options: curl -sSfL https://zyanya.org/install.sh | bash -s -- --testnet
# ==============================================================================
set -euo pipefail

BOLD='\033[1m'
GREEN='\033[0;32m'
CYAN='\033[0;36m'
YELLOW='\033[1;33m'
RED='\033[0;31m'
PURPLE='\033[0;35m'
RESET='\033[0m'

SEEDS_IPV6=(
    "2600:1f13:955:2001:5a4a:c577:fa93:6e6f"
    "2600:1f13:955:2001:99a1:b6b4:46d8:53f4"
)
PORT_P2P_MAINNET=18111
PORT_RPC_MAINNET=18110
PORT_P2P_TESTNET=18211
PORT_RPC_TESTNET=18210

NETWORK="mainnet"
INSTALL_DIR="/usr/local/bin"
CONF_DIR="${HOME}/.zyanyad"
SERVICE_NAME="zyanyad"
SKIP_SERVICE=false

# Parse args
while [[ $# -gt 0 ]]; do
    case "$1" in
        --testnet|--testnet-10)
            NETWORK="testnet-10"
            shift
            ;;
        --no-service)
            SKIP_SERVICE=true
            shift
            ;;
        --prefix)
            INSTALL_DIR="$2"
            shift 2
            ;;
        *)
            shift
            ;;
    esac
done

echo -e "${CYAN}${BOLD}"
echo "  ______                                     "
echo " |___  /                                     "
echo "    / / _   _   __ _  _ __   _   _   __ _    "
echo "   / / | | | | / _\` || '_ \ | | | | / _\` |   "
echo "  / /__| |_| || (_| || | | || |_| || (_| |   "
echo " /_____|\__, | \__,_||_| |_| \__, | \__,_|   "
echo "         __/ |                __/ |          "
echo "        |___/                |___/           "
echo -e "${RESET}"
echo -e "${BOLD}Zyanya GhostDAG L1 Headless Installer & Node Provisioner${RESET}"
echo -e "Target Network: ${PURPLE}${NETWORK}${RESET}"
echo "--------------------------------------------------------"

# 1. Detect Architecture & OS
OS="$(uname -s | tr '[:upper:]' '[:lower:]')"
ARCH="$(uname -m)"

case "$ARCH" in
    x86_64|amd64)
        ARCH="x86_64"
        ;;
    aarch64|arm64)
        ARCH="aarch64"
        ;;
    *)
        echo -e "${RED}[!] Unsupported architecture: ${ARCH}${RESET}"
        exit 1
        ;;
esac

echo -e "${GREEN}[✓] Detected System:${RESET} ${OS} (${ARCH})"

# 2. Ghost Pinhole IPv6 Setup & Diagnostic Assistant
echo ""
echo "--------------------------------------------------------"
echo -e "${BOLD}🦅 Ghost Pinhole IPv6 Setup & Diagnostic Assistant${RESET}"
echo "--------------------------------------------------------"

IPV6_GLOBAL=""
if command -v ip >/dev/null 2>&1; then
    IPV6_GLOBAL=$(ip -6 addr show scope global | grep -oE 'inet6 [0-9a-fA-F:]+' | awk '{print $2}' | head -n 1 || true)
elif command -v ifconfig >/dev/null 2>&1; then
    IPV6_GLOBAL=$(ifconfig | grep -oE 'inet6 [0-9a-fA-F:]+' | grep -v 'fe80:' | awk '{print $2}' | head -n 1 || true)
fi

if [ -n "$IPV6_GLOBAL" ]; then
    echo -e "${GREEN}[✓] Local Global IPv6 Address Detected:${RESET} ${IPV6_GLOBAL}"
else
    echo -e "${YELLOW}[!] No global IPv6 address detected on local interfaces.${RESET}"
    echo -e "    Zyanya uses IPv6-first GhostDAG peering for direct point-to-point block dissemination."
fi

# Test probe to Canonical Seeds
SEED_OK=false
TARGET_PORT=$PORT_P2P_MAINNET
if [ "$NETWORK" = "testnet-10" ]; then
    TARGET_PORT=$PORT_P2P_TESTNET
fi

echo -e "${CYAN}[*] Testing connection to canonical Zyanya IPv6 seed nodes (Port ${TARGET_PORT})...${RESET}"
for SEED in "${SEEDS_IPV6[@]}"; do
    # Use nc or bash built-in /dev/tcp if available
    if command -v nc >/dev/null 2>&1; then
        if nc -6 -z -w 3 "$SEED" "$TARGET_PORT" 2>/dev/null; then
            echo -e "${GREEN}[✓] Seed Reachable:${RESET} [${SEED}]:${TARGET_PORT}"
            SEED_OK=true
            break
        fi
    fi
done

if [ "$SEED_OK" = true ]; then
    echo -e "${GREEN}[✓] Outbound IPv6 GhostDAG mesh path is active and verified!${RESET}"
else
    echo -e "${YELLOW}[i] Outbound probe to seed node timed out or port closed.${RESET}"
    echo -e "${BOLD}--- Ghost Pinhole Advisory ---${RESET}"
    echo -e "  Most residential ISPs delegate full IPv6 prefixes (/64 or /60), but home routers"
    echo -e "  (e.g., Comcast, AT&T, Asus, Netgear, eero) block unsolicited inbound packets."
    echo -e "  To ensure maximum block propagation and earn mining peer rewards:"
    echo -e "  1. Open your router's admin interface (e.g. 192.168.1.1 or 10.0.0.1)."
    echo -e "  2. Navigate to ${BOLD}Firewall -> IPv6 Pinholes / Custom Rules${RESET}."
    echo -e "  3. Allow inbound ${BOLD}TCP port ${TARGET_PORT}${RESET} to your device's IPv6: ${IPV6_GLOBAL:-[Your Device IPv6]}."
    echo -e "  ${CYAN}(Note: IPv6 does not require NAT port-forwarding; only a firewall pinhole!)${RESET}"
fi
echo "--------------------------------------------------------"

# 3. Directory and Permissions Check
if [ "$EUID" -ne 0 ]; then
    if [ ! -w "$INSTALL_DIR" ]; then
        INSTALL_DIR="${HOME}/.local/bin"
        mkdir -p "$INSTALL_DIR"
        echo -e "${YELLOW}[!] No write permission to /usr/local/bin. Installing to ${INSTALL_DIR}${RESET}"
        if [[ ":$PATH:" != *":$INSTALL_DIR:"* ]]; then
            echo "export PATH=\"\$PATH:${INSTALL_DIR}\"" >> "${HOME}/.bashrc"
            export PATH="$PATH:${INSTALL_DIR}"
            echo -e "${GREEN}[✓] Added ${INSTALL_DIR} to PATH in ~/.bashrc${RESET}"
        fi
    fi
fi

# 4. Binary Installation
echo ""
echo -e "${CYAN}[*] Provisioning Zyanya GhostDAG binaries...${RESET}"
RELEASE_BASE="https://github.com/scotthawk-maker/zyanya/releases/latest/download"
TARBALL="zyanya-${OS}-${ARCH}.tar.gz"
TEMP_DIR="$(mktemp -d)"

DOWNLOAD_SUCCESS=false
if command -v curl >/dev/null 2>&1; then
    if curl -sSfL -o "${TEMP_DIR}/${TARBALL}" "${RELEASE_BASE}/${TARBALL}" 2>/dev/null; then
        DOWNLOAD_SUCCESS=true
    fi
fi

if [ "$DOWNLOAD_SUCCESS" = true ]; then
    echo -e "${GREEN}[✓] Downloaded release bundle: ${TARBALL}${RESET}"
    tar -xzf "${TEMP_DIR}/${TARBALL}" -C "${TEMP_DIR}"
    install -m 755 "${TEMP_DIR}/zyanyad" "${INSTALL_DIR}/zyanyad"
    install -m 755 "${TEMP_DIR}/zyanya-cli" "${INSTALL_DIR}/zyanya-cli"
    [ -f "${TEMP_DIR}/zyanya-miner" ] && install -m 755 "${TEMP_DIR}/zyanya-miner" "${INSTALL_DIR}/zyanya-miner"
else
    echo -e "${YELLOW}[i] Pre-built binary package not found on latest GitHub release.${RESET}"
    LOCAL_BUILD="${ZYANYA_LOCAL_BUILD:-../../target/release}"
    if [ -f "${LOCAL_BUILD}/zyanyad" ]; then
        echo -e "${GREEN}[✓] Found locally compiled binaries in ${LOCAL_BUILD}! Copying...${RESET}"
        cp "${LOCAL_BUILD}/zyanyad" "${INSTALL_DIR}/zyanyad"
        cp "${LOCAL_BUILD}/zyanya-cli" "${INSTALL_DIR}/zyanya-cli"
        [ -f "${LOCAL_BUILD}/zyanya-miner" ] && cp "${LOCAL_BUILD}/zyanya-miner" "${INSTALL_DIR}/zyanya-miner"
        chmod +x "${INSTALL_DIR}/zyanyad" "${INSTALL_DIR}/zyanya-cli"
    elif command -v cargo >/dev/null 2>&1; then
        echo -e "${CYAN}[*] Building from source using cargo...${RESET}"
        git clone --depth 1 https://github.com/scotthawk-maker/zyanya.git "${TEMP_DIR}/zyanya-src"
        cd "${TEMP_DIR}/zyanya-src"
        cargo build --release --bin zyanyad --bin zyanya-cli --bin zyanya-miner
        install -m 755 target/release/zyanyad "${INSTALL_DIR}/zyanyad"
        install -m 755 target/release/zyanya-cli "${INSTALL_DIR}/zyanya-cli"
        install -m 755 target/release/zyanya-miner "${INSTALL_DIR}/zyanya-miner"
    else
        echo -e "${RED}[!] Could not locate pre-built binaries or cargo toolchain.${RESET}"
        echo -e "    Please install Rust via 'curl --proto =https --tlsv1.2 -sSf https://sh.rustup.rs | sh'"
        rm -rf "${TEMP_DIR}"
        exit 1
    fi
fi
rm -rf "${TEMP_DIR}"

echo -e "${GREEN}[✓] Installed binaries:${RESET}"
echo "    - ${INSTALL_DIR}/zyanyad"
echo "    - ${INSTALL_DIR}/zyanya-cli"
[ -f "${INSTALL_DIR}/zyanya-miner" ] && echo "    - ${INSTALL_DIR}/zyanya-miner"

# 5. Configuration Setup
mkdir -p "${CONF_DIR}"
CONF_FILE="${CONF_DIR}/zyanya.conf"

if [ ! -f "${CONF_FILE}" ]; then
    cat <<EOF > "${CONF_FILE}"
# Zyanya GhostDAG L1 Configuration File
# Auto-generated by Zyanya Headless Installer

appdir=${CONF_DIR}/data
listen=[::]:${TARGET_PORT}
rpclisten=[::1]:$([ "$NETWORK" = "testnet-10" ] && echo $PORT_RPC_TESTNET || echo $PORT_RPC_MAINNET)
$([ "$NETWORK" = "testnet-10" ] && echo "testnet=1" || echo "# mainnet=1")

# Canonical IPv6 Seeds
addpeer=[2600:1f13:955:2001:5a4a:c577:fa93:6e6f]:${TARGET_PORT}
addpeer=[2600:1f13:955:2001:99a1:b6b4:46d8:53f4]:${TARGET_PORT}

# Performance & Logging
logdir=${CONF_DIR}/logs
loglevel=info
EOF
    echo -e "${GREEN}[✓] Created configuration at ${CONF_FILE}${RESET}"
fi

# 6. Systemd Service Configuration
if [ "$SKIP_SERVICE" = false ] && command -v systemctl >/dev/null 2>&1; then
    if [ "$EUID" -eq 0 ]; then
        SERVICE_PATH="/etc/systemd/system/${SERVICE_NAME}.service"
        cat <<EOF > "${SERVICE_PATH}"
[Unit]
Description=Zyanya GhostDAG L1 Node Daemon
After=network.target network-online.target
Wants=network-online.target

[Service]
Type=simple
User=${SUDO_USER:-root}
ExecStart=${INSTALL_DIR}/zyanyad --configfile=${CONF_FILE}
Restart=always
RestartSec=10
LimitNOFILE=65535

[Install]
WantedBy=multi-user.target
EOF
        systemctl daemon-reload
        systemctl enable "${SERVICE_NAME}"
        systemctl start "${SERVICE_NAME}"
        echo -e "${GREEN}[✓] Configured and started systemd service: ${SERVICE_NAME}.service${RESET}"
    else
        # User service
        USER_SERVICE_DIR="${HOME}/.config/systemd/user"
        mkdir -p "${USER_SERVICE_DIR}"
        cat <<EOF > "${USER_SERVICE_DIR}/${SERVICE_NAME}.service"
[Unit]
Description=Zyanya GhostDAG L1 Node Daemon (User)
After=network.target

[Service]
Type=simple
ExecStart=${INSTALL_DIR}/zyanyad --configfile=${CONF_FILE}
Restart=always
RestartSec=10
LimitNOFILE=65535

[Install]
WantedBy=default.target
EOF
        systemctl --user daemon-reload || true
        systemctl --user enable "${SERVICE_NAME}" || true
        systemctl --user start "${SERVICE_NAME}" || true
        echo -e "${GREEN}[✓] Configured and started user systemd service: ${SERVICE_NAME}.service${RESET}"
    fi
fi

echo ""
echo "========================================================"
echo -e "${GREEN}${BOLD}🎉 Zyanya GhostDAG L1 Node Successfully Installed!${RESET}"
echo "========================================================"
echo -e "• Check node status:      ${CYAN}${INSTALL_DIR}/zyanya-cli get-info${RESET}"
echo -e "• Check peer connections: ${CYAN}${INSTALL_DIR}/zyanya-cli get-connected-peer-info${RESET}"
echo -e "• Check DAG sync metrics: ${CYAN}${INSTALL_DIR}/zyanya-cli get-block-dag-info${RESET}"
echo -e "• Configuration path:     ${CYAN}${CONF_FILE}${RESET}"
echo "========================================================"
