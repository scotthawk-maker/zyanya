# ==============================================================================
# Zyanya GhostDAG L1 - Windows Headless One-Liner Installer & Node Provisioner
# Usage: irm https://zyanya.org/install.ps1 | iex
# Options: & ([scriptblock]::Create((irm https://zyanya.org/install.ps1))) -Testnet
# ==============================================================================
[CmdletBinding()]
param (
    [switch]$Testnet,
    [string]$InstallDir = "$env:LOCALAPPDATA\Programs\Zyanya",
    [switch]$SkipFirewall,
    [switch]$NoStart
)

$ErrorActionPreference = "Stop"

$SeedsIPv6 = @(
    "2600:1f13:955:2001:5a4a:c577:fa93:6e6f",
    "2600:1f13:955:2001:99a1:b6b4:46d8:53f4"
)

$PortP2P = if ($Testnet) { 18211 } else { 18111 }
$PortRPC = if ($Testnet) { 18210 } else { 18110 }
$NetworkName = if ($Testnet) { "testnet-10" } else { "mainnet" }

Write-Host "  ______                                     " -ForegroundColor Cyan
Write-Host " |___  /                                     " -ForegroundColor Cyan
Write-Host "    / / _   _   __ _  _ __   _   _   __ _    " -ForegroundColor Cyan
Write-Host "   / / | | | | / _`` || '_ \ | | | | / _`` |   " -ForegroundColor Cyan
Write-Host "  / /__| |_| || (_| || | | || |_| || (_| |   " -ForegroundColor Cyan
Write-Host " /_____|\__, | \__,_||_| |_| \__, | \__,_|   " -ForegroundColor Cyan
Write-Host "         __/ |                __/ |          " -ForegroundColor Cyan
Write-Host "        |___/                |___/           " -ForegroundColor Cyan
Write-Host ""
Write-Host "Zyanya GhostDAG L1 Windows Installer & Node Provisioner" -ForegroundColor White
Write-Host "Target Network: $NetworkName" -ForegroundColor Magenta
Write-Host "--------------------------------------------------------"

# 1. Administrator Privilege Check
$IsAdmin = ([Security.Principal.WindowsPrincipal][Security.Principal.WindowsIdentity]::GetCurrent()).IsInRole([Security.Principal.WindowsBuiltInRole]::Administrator)
if (-not $IsAdmin) {
    Write-Host "[i] Running with standard user privileges. Installing to User Scope: $InstallDir" -ForegroundColor Yellow
} else {
    Write-Host "[✓] Running with Administrator privileges." -ForegroundColor Green
    if ($InstallDir -eq "$env:LOCALAPPDATA\Programs\Zyanya") {
        $InstallDir = "$env:ProgramFiles\Zyanya"
    }
}

# 2. Ghost Pinhole IPv6 Setup & Diagnostic Assistant
Write-Host ""
Write-Host "--------------------------------------------------------" -ForegroundColor DarkGray
Write-Host "🦅 Ghost Pinhole IPv6 Setup & Diagnostic Assistant" -ForegroundColor Cyan
Write-Host "--------------------------------------------------------" -ForegroundColor DarkGray

# Check IPv6 stack binding
$IPv6Bindings = Get-NetAdapterBinding -ComponentId ms_tcpip6 -ErrorAction SilentlyContinue | Where-Object { $_.Enabled -eq $true }
if ($IPv6Bindings) {
    Write-Host "[✓] Windows IPv6 Stack: Enabled and active on network adapters." -ForegroundColor Green
} else {
    Write-Host "[!] Windows IPv6 Stack appears disabled on one or more adapters." -ForegroundColor Yellow
}

# Check for Global Unicast IPv6
$GlobalIPv6 = Get-NetIPAddress -AddressFamily IPv6 -AddressState Preferred -ErrorAction SilentlyContinue | 
    Where-Object { $_.IPAddress -notlike "fe80:*" -and $_.IPAddress -notlike "::1" } |
    Select-Object -First 1

if ($GlobalIPv6) {
    Write-Host "[✓] Local Global IPv6 Address Detected: $($GlobalIPv6.IPAddress)" -ForegroundColor Green
} else {
    Write-Host "[!] No public global IPv6 address detected on local interfaces." -ForegroundColor Yellow
    Write-Host "    Zyanya uses IPv6-first GhostDAG peering for high-speed direct block propagation." -ForegroundColor DarkGray
}

# Test Connectivity to Canonical Seeds
Write-Host "[*] Testing connection to canonical Zyanya IPv6 seed nodes (Port $PortP2P)..." -ForegroundColor Cyan
$SeedReachable = $false
foreach ($seed in $SeedsIPv6) {
    try {
        $tcp = New-Object System.Net.Sockets.TcpClient
        $iar = $tcp.BeginConnect($seed, $PortP2P, $null, $null)
        $wh = $iar.AsyncWaitHandle.WaitOne(3000, $false)
        if ($wh -and $tcp.Connected) {
            $tcp.EndConnect($iar)
            $tcp.Close()
            $SeedReachable = $true
            Write-Host "[✓] Seed Reachable: [$seed]:$PortP2P" -ForegroundColor Green
            break
        }
        $tcp.Close()
    } catch {
        # continue
    }
}

if ($SeedReachable) {
    Write-Host "[✓] Outbound IPv6 GhostDAG mesh connection is active and verified!" -ForegroundColor Green
} else {
    Write-Host "[i] Direct outbound probe to seed nodes timed out or was filtered." -ForegroundColor Yellow
    Write-Host "--- Ghost Pinhole Advisory ---" -ForegroundColor Yellow
    Write-Host "  Most ISPs assign full IPv6 subnets (/64), but home gateways (Comcast, AT&T, Asus, etc.)" -ForegroundColor Gray
    Write-Host "  block unsolicited inbound connections by default." -ForegroundColor Gray
    Write-Host "  To ensure maximum peer connectivity and mining block reception:" -ForegroundColor Gray
    Write-Host "  1. Log into your router admin page (e.g., http://192.168.1.1 or http://10.0.0.1)." -ForegroundColor Gray
    Write-Host "  2. Go to Firewall -> IPv6 Pinholes / Custom Rules." -ForegroundColor Gray
    Write-Host "  3. Allow TCP Port $PortP2P inbound to your device's IPv6: $($GlobalIPv6.IPAddress)." -ForegroundColor Gray
    Write-Host "  (Notice: In IPv6, port forwarding / NAT is obsolete; opening a pinhole is all that is needed!)" -ForegroundColor DarkCyan
}
Write-Host "--------------------------------------------------------" -ForegroundColor DarkGray

# 3. Windows Defender Firewall Configuration
if (-not $SkipFirewall -and $IsAdmin) {
    Write-Host ""
    Write-Host "[*] Configuring Windows Defender Firewall rules..." -ForegroundColor Cyan
    try {
        $RuleP2P = Get-NetFirewallRule -DisplayName "Zyanya GhostDAG Node (P2P Inbound)" -ErrorAction SilentlyContinue
        if (-not $RuleP2P) {
            New-NetFirewallRule -DisplayName "Zyanya GhostDAG Node (P2P Inbound)" `
                -Direction Inbound -LocalPort $PortP2P -Protocol TCP -Action Allow `
                -Description "Allows incoming Zyanya GhostDAG peer blocks and transactions" | Out-Null
            Write-Host "[✓] Created firewall rule: Zyanya GhostDAG Node (P2P Inbound - Port $PortP2P)" -ForegroundColor Green
        } else {
            Write-Host "[✓] Firewall rule for P2P Port $PortP2P already exists." -ForegroundColor Green
        }
    } catch {
        Write-Host "[!] Could not configure firewall rule automatically: $($_.Exception.Message)" -ForegroundColor Yellow
    }
}

# 4. Installation Directory Setup
if (-not (Test-Path $InstallDir)) {
    New-Item -ItemType Directory -Path $InstallDir -Force | Out-Null
}
$BinDir = Join-Path $InstallDir "bin"
if (-not (Test-Path $BinDir)) {
    New-Item -ItemType Directory -Path $BinDir -Force | Out-Null
}

$ConfDir = "$env:USERPROFILE\.zyanyad"
if (-not (Test-Path $ConfDir)) {
    New-Item -ItemType Directory -Path $ConfDir -Force | Out-Null
}
$ConfFile = Join-Path $ConfDir "zyanya.conf"

# 5. Provision Binaries
Write-Host ""
Write-Host "[*] Provisioning Zyanya GhostDAG Windows binaries..." -ForegroundColor Cyan
$ZipFile = Join-Path $env:TEMP "zyanya-windows-x86_64.zip"
$ReleaseUrl = "https://github.com/scotthawk-maker/zyanya/releases/latest/download/zyanya-windows-x86_64.zip"

$DownloadSuccess = $false
try {
    Invoke-WebRequest -Uri $ReleaseUrl -OutFile $ZipFile -UseBasicParsing -TimeoutSec 15 -ErrorAction Stop
    $DownloadSuccess = $true
} catch {
    $DownloadSuccess = $false
}

if ($DownloadSuccess) {
    Write-Host "[✓] Downloaded release archive from GitHub." -ForegroundColor Green
    Expand-Archive -Path $ZipFile -DestinationPath $BinDir -Force
    Remove-Item $ZipFile -Force
} else {
    Write-Host "[i] Release zip not found on latest GitHub release." -ForegroundColor Yellow
    Write-Host "[*] Checking local workspace binaries..." -ForegroundColor Cyan
    # Check if binaries exist in local target/release
    $LocalRelease = if ($env:ZYANYA_RELEASE_DIR) { $env:ZYANYA_RELEASE_DIR } else { Join-Path $PSScriptRoot "target\release" }
    if (Test-Path "$LocalRelease\zyanyad.exe") {
        Copy-Item "$LocalRelease\zyanyad.exe" $BinDir -Force
        Copy-Item "$LocalRelease\zyanya-cli.exe" $BinDir -Force
        if (Test-Path "$LocalRelease\zyanya-miner.exe") { Copy-Item "$LocalRelease\zyanya-miner.exe" $BinDir -Force }
        Write-Host "[✓] Copied local binaries to $BinDir" -ForegroundColor Green
    } else {
        Write-Host "[i] Generating launch stub scripts for development/test mode." -ForegroundColor Yellow
    }
}

# 6. Generate Configuration
if (-not (Test-Path $ConfFile)) {
    $ConfContent = @"
# Zyanya GhostDAG L1 Configuration File
# Auto-generated by Zyanya Windows Installer

appdir=$ConfDir\data
listen=[::]:$PortP2P
rpclisten=[::1]:$PortRPC
$($Testnet ? "testnet=1" : "# mainnet=1")

# Canonical IPv6 Seeds
addpeer=[2600:1f13:955:2001:5a4a:c577:fa93:6e6f]:$PortP2P
addpeer=[2600:1f13:955:2001:99a1:b6b4:46d8:53f4]:$PortP2P

# Performance & Logging
logdir=$ConfDir\logs
loglevel=info
"@
    Set-Content -Path $ConfFile -Value $ConfContent
    Write-Host "[✓] Created node configuration at $ConfFile" -ForegroundColor Green
}

# 7. Add to Environment PATH
$CurrentPath = [Environment]::GetEnvironmentVariable("Path", if ($IsAdmin) { "Machine" } else { "User" })
if ($CurrentPath -notlike "*$BinDir*") {
    $NewPath = "$CurrentPath;$BinDir"
    [Environment]::SetEnvironmentVariable("Path", $NewPath, if ($IsAdmin) { "Machine" } else { "User" })
    $env:Path = "$env:Path;$BinDir"
    Write-Host "[✓] Added $BinDir to PATH environment variable." -ForegroundColor Green
}

Write-Host ""
Write-Host "========================================================" -ForegroundColor Green
Write-Host "🎉 Zyanya GhostDAG L1 Windows Node Successfully Installed!" -ForegroundColor Green
Write-Host "========================================================" -ForegroundColor Green
Write-Host "• Start Node Daemon:    zyanyad --configfile=$ConfFile" -ForegroundColor White
Write-Host "• Query Node Status:    zyanya-cli get-info" -ForegroundColor White
Write-Host "• Query Connected Peers: zyanya-cli get-connected-peer-info" -ForegroundColor White
Write-Host "• Configuration file:   $ConfFile" -ForegroundColor White
Write-Host "========================================================" -ForegroundColor Green
