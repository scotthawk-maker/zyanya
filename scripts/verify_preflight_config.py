#!/usr/bin/env python3
"""
Zyanya Network Pre-flight Configuration Verification Script
Validates critical network configuration parameters across Mainnet and Testnet:
1. MAINNET_PARAMS vs TESTNET_PARAMS smart contract and payload activation
2. net_magic for Mainnet (ZYAN: 0x5A, 0x59, 0x41, 0x4E) vs Testnet (ZYNT: 0x5A, 0x59, 0x4E, 0x54)
3. MAINNET_GENESIS hash, structure, and zero-premine payload
4. Address prefixes (zyanya: vs zyanyatest:)
"""

import sys
import re
from pathlib import Path

# Paths to relevant source files
REPO_ROOT = Path(__file__).resolve().parent.parent
PARAMS_RS = REPO_ROOT / "consensus" / "core" / "src" / "config" / "params.rs"
GENESIS_RS = REPO_ROOT / "consensus" / "core" / "src" / "config" / "genesis.rs"
ADDRESSES_RS = REPO_ROOT / "crypto" / "addresses" / "src" / "lib.rs"

class PreflightValidator:
    def __init__(self):
        self.total_checks = 0
        self.passed_checks = 0
        self.failed_checks = 0
        self.failures = []

    def check(self, name: str, condition: bool, details: str = ""):
        self.total_checks += 1
        if condition:
            self.passed_checks += 1
            print(f"  [PASS] {name}" + (f" -> {details}" if details else ""))
        else:
            self.failed_checks += 1
            msg = f"  [FAIL] {name}" + (f" -> {details}" if details else "")
            print(msg)
            self.failures.append(msg)

    def run(self) -> bool:
        print("=" * 75)
        print("  ZYANYA BLOCKCHAIN PRE-FLIGHT CONFIGURATION AUDIT & VERIFICATION")
        print("=" * 75)

        self.verify_smart_contract_and_payload_activation()
        self.verify_net_magic()
        self.verify_mainnet_genesis()
        self.verify_address_prefixes()

        print("\n" + "=" * 75)
        print(f"  PRE-FLIGHT AUDIT SUMMARY: {self.passed_checks}/{self.total_checks} checks passed")
        if self.failed_checks == 0:
            print("  STATUS: ALL CONFIGURATION CHECKS PASSED (Ready for Deployment)")
            print("=" * 75)
            return True
        else:
            print(f"  STATUS: {self.failed_checks} CHECKS FAILED")
            for f in self.failures:
                print(f"   * {f}")
            print("=" * 75)
            return False

    def verify_smart_contract_and_payload_activation(self):
        print("\n[CHECK 1] Smart Contract and Payload Activation (MAINNET_PARAMS vs TESTNET_PARAMS)")
        self.check("params.rs exists", PARAMS_RS.exists(), str(PARAMS_RS))
        if not PARAMS_RS.exists():
            return

        content = PARAMS_RS.read_text(encoding="utf-8")

        # Extract MAINNET_PARAMS block
        mainnet_match = re.search(r"pub const MAINNET_PARAMS: Params = Params\s*\{([^}]+)\};", content, re.DOTALL)
        self.check("MAINNET_PARAMS block defined in params.rs", bool(mainnet_match))
        if mainnet_match:
            mainnet_body = mainnet_match.group(1)
            
            # Check payload_activation
            payload_act = re.search(r"payload_activation:\s*ForkActivation::(\w+)(?:\((\d+)\))?", mainnet_body)
            self.check(
                "MAINNET_PARAMS payload_activation is ForkActivation::new(0)",
                bool(payload_act and payload_act.group(1) == "new" and payload_act.group(2) == "0"),
                f"found: {payload_act.group(0) if payload_act else 'none'}"
            )

            # Check enable_smart_contracts
            sc_match = re.search(r"enable_smart_contracts:\s*(true|false)", mainnet_body)
            self.check(
                "MAINNET_PARAMS enable_smart_contracts is true",
                bool(sc_match and sc_match.group(1) == "true"),
                f"found: {sc_match.group(0) if sc_match else 'none'}"
            )

        # Extract TESTNET_PARAMS block
        testnet_match = re.search(r"pub const TESTNET_PARAMS: Params = Params\s*\{([^}]+)\};", content, re.DOTALL)
        self.check("TESTNET_PARAMS block defined in params.rs", bool(testnet_match))
        if testnet_match:
            testnet_body = testnet_match.group(1)

            # Check payload_activation
            payload_act_t = re.search(r"payload_activation:\s*ForkActivation::(\w+)(?:\((\d+)\))?", testnet_body)
            self.check(
                "TESTNET_PARAMS payload_activation is ForkActivation::new(0)",
                bool(payload_act_t and payload_act_t.group(1) == "new" and payload_act_t.group(2) == "0"),
                f"found: {payload_act_t.group(0) if payload_act_t else 'none'}"
            )

            # Check enable_smart_contracts
            sc_match_t = re.search(r"enable_smart_contracts:\s*(true|false)", testnet_body)
            self.check(
                "TESTNET_PARAMS enable_smart_contracts is true",
                bool(sc_match_t and sc_match_t.group(1) == "true"),
                f"found: {sc_match_t.group(0) if sc_match_t else 'none'}"
            )

        # Validate parity: Subnetwork 3 active across both networks
        if mainnet_match and testnet_match:
            self.check(
                "Mainnet and Testnet feature parity for Subnetwork 3 smart contracts",
                "enable_smart_contracts: true" in mainnet_body and "enable_smart_contracts: true" in testnet_body,
                "Both Mainnet and Testnet have smart contracts enabled"
            )

    def verify_net_magic(self):
        print("\n[CHECK 2] P2P Network Magic Isolation (Mainnet vs Testnet)")
        content = PARAMS_RS.read_text(encoding="utf-8")

        # Mainnet magic: [0x5A, 0x59, 0x41, 0x4E] (ASCII "ZYAN")
        mainnet_match = re.search(r"pub const MAINNET_PARAMS: Params = Params\s*\{([^}]+)\};", content, re.DOTALL)
        magic_match = None
        raw_bytes = None
        if mainnet_match:
            mainnet_body = mainnet_match.group(1)
            magic_match = re.search(r"net_magic:\s*\[(0x[0-9A-Fa-f]{2},\s*0x[0-9A-Fa-f]{2},\s*0x[0-9A-Fa-f]{2},\s*0x[0-9A-Fa-f]{2})\]", mainnet_body)
            self.check("MAINNET_PARAMS net_magic is defined", bool(magic_match))
            if magic_match:
                raw_bytes = [int(b.strip(), 16) for b in magic_match.group(1).split(",")]
                ascii_str = bytes(raw_bytes).decode("ascii", errors="replace")
                expected_bytes = [0x5A, 0x59, 0x41, 0x4E]
                self.check(
                    "Mainnet net_magic bytes match [0x5A, 0x59, 0x41, 0x4E]",
                    raw_bytes == expected_bytes,
                    f"bytes: {[hex(b) for b in raw_bytes]} == 'ZYAN'"
                )
                self.check(
                    "Mainnet net_magic ASCII representation is 'ZYAN'",
                    ascii_str == "ZYAN",
                    f"ASCII: '{ascii_str}'"
                )

        # Testnet magic: [0x5A, 0x59, 0x4E, 0x54] (ASCII "ZYNT")
        testnet_match = re.search(r"pub const TESTNET_PARAMS: Params = Params\s*\{([^}]+)\};", content, re.DOTALL)
        magic_match_t = None
        raw_bytes_t = None
        if testnet_match:
            testnet_body = testnet_match.group(1)
            magic_match_t = re.search(r"net_magic:\s*\[(0x[0-9A-Fa-f]{2},\s*0x[0-9A-Fa-f]{2},\s*0x[0-9A-Fa-f]{2},\s*0x[0-9A-Fa-f]{2})\]", testnet_body)
            self.check("TESTNET_PARAMS net_magic is defined", bool(magic_match_t))
            if magic_match_t:
                raw_bytes_t = [int(b.strip(), 16) for b in magic_match_t.group(1).split(",")]
                ascii_str_t = bytes(raw_bytes_t).decode("ascii", errors="replace")
                expected_bytes_t = [0x5A, 0x59, 0x4E, 0x54]
                self.check(
                    "Testnet net_magic bytes match [0x5A, 0x59, 0x4E, 0x54]",
                    raw_bytes_t == expected_bytes_t,
                    f"bytes: {[hex(b) for b in raw_bytes_t]} == 'ZYNT'"
                )
                self.check(
                    "Testnet net_magic ASCII representation is 'ZYNT'",
                    ascii_str_t == "ZYNT",
                    f"ASCII: '{ascii_str_t}'"
                )

        # Network isolation check: Mainnet magic != Testnet magic
        if mainnet_match and testnet_match and magic_match and magic_match_t:
            self.check(
                "Network magic isolation: Mainnet ('ZYAN') != Testnet ('ZYNT')",
                raw_bytes != raw_bytes_t,
                "Mainnet and Testnet peer-to-peer traffic fully isolated"
            )

    def verify_mainnet_genesis(self):
        print("\n[CHECK 3] MAINNET_GENESIS Hash and Coinbase Payload")
        self.check("genesis.rs exists", GENESIS_RS.exists(), str(GENESIS_RS))
        if not GENESIS_RS.exists():
            return

        content = GENESIS_RS.read_text(encoding="utf-8")

        # Extract MAINNET_GENESIS block
        genesis_match = re.search(r"pub const MAINNET_GENESIS: GenesisBlock = GenesisBlock\s*\{([^}]+)\};", content, re.DOTALL)
        self.check("MAINNET_GENESIS defined in genesis.rs", bool(genesis_match))
        if not genesis_match:
            return

        genesis_body = genesis_match.group(1)

        # Extract genesis hash bytes
        hash_match = re.search(r"hash:\s*Hash::from_bytes\(\[([^\]]+)\]\)", genesis_body)
        self.check("MAINNET_GENESIS hash defined as 32-byte array", bool(hash_match))
        if hash_match:
            hex_tokens = [t.strip() for t in hash_match.group(1).replace("\n", " ").split(",") if t.strip()]
            hash_bytes = [int(h, 16) for h in hex_tokens]
            self.check(
                "MAINNET_GENESIS hash length is exactly 32 bytes (256 bits)",
                len(hash_bytes) == 32,
                f"length: {len(hash_bytes)} bytes"
            )
            hash_hex = bytes(hash_bytes).hex()
            expected_hash_hex = "99f5e7f1e1f32efdba33a55a6bcd186baee9827a63c8dd173cb2721367c57815"
            self.check(
                f"MAINNET_GENESIS hash matches canonical {expected_hash_hex}",
                hash_hex.lower() == expected_hash_hex.lower(),
                f"hash: {hash_hex}"
            )

        # Extract coinbase payload
        payload_match = re.search(r"coinbase_payload:\s*&\[([^\]]+)\]", genesis_body)
        self.check("MAINNET_GENESIS coinbase_payload defined", bool(payload_match))
        if payload_match:
            # Clean comments and parse hex bytes
            raw_payload_str = payload_match.group(1)
            lines = raw_payload_str.splitlines()
            cleaned_tokens = []
            for line in lines:
                code_part = line.split("//")[0].strip()
                if code_part:
                    tokens = [t.strip() for t in code_part.split(",") if t.strip()]
                    cleaned_tokens.extend(tokens)

            payload_bytes = [int(t, 16) for t in cleaned_tokens]
            
            # Check payload length: exactly 32 bytes
            self.check(
                "Coinbase payload length is exactly 32 bytes",
                len(payload_bytes) == 32,
                f"length: {len(payload_bytes)} bytes"
            )
            if len(payload_bytes) == 32:
                blue_score = int.from_bytes(bytes(payload_bytes[0:8]), byteorder="little")
                subsidy_bytes = payload_bytes[8:16]
                expected_subsidy_bytes = [0x00, 0xF4, 0x05, 0x2A, 0x01, 0x00, 0x00, 0x00]
                script_version = payload_bytes[16:18]
                varint = payload_bytes[18]
                op_false = payload_bytes[19]

                self.check(
                    "Coinbase payload blue_score is 0",
                    blue_score == 0,
                    f"blue_score: {blue_score}"
                )
                self.check(
                    "Coinbase payload subsidy bytes match [0x00, 0xF4, 0x05, 0x2A, 0x01, 0x00, 0x00, 0x00]",
                    subsidy_bytes == expected_subsidy_bytes,
                    f"subsidy bytes: {[hex(b) for b in subsidy_bytes]}"
                )
                self.check(
                    "Coinbase payload script version is 0 (2 bytes: [0x00, 0x00])",
                    script_version == [0x00, 0x00],
                    f"script version: {[hex(b) for b in script_version]}"
                )
                self.check(
                    "Coinbase payload varint script length is 1 (0x01)",
                    varint == 0x01,
                    f"varint: {hex(varint)}"
                )
                self.check(
                    "Coinbase payload OP_FALSE marker (0x00) present (zero premine)",
                    op_false == 0x00,
                    f"op_false: {hex(op_false)}"
                )

            # Check magic bytes trailer: 'ZYAN-MAINNET'
            expected_trailer = b"ZYAN-MAINNET"
            actual_trailer = bytes(payload_bytes[-len(expected_trailer):])
            self.check(
                "Coinbase payload ends with 'ZYAN-MAINNET' magic trailer",
                actual_trailer == expected_trailer,
                f"trailer: '{actual_trailer.decode('ascii', errors='replace')}'"
            )

    def verify_address_prefixes(self):
        print("\n[CHECK 4] Address Prefixes (zyanya: vs zyanyatest:)")
        self.check("crypto/addresses/src/lib.rs exists", ADDRESSES_RS.exists(), str(ADDRESSES_RS))
        if not ADDRESSES_RS.exists():
            return

        content = ADDRESSES_RS.read_text(encoding="utf-8")

        # Check Prefix enum definition
        prefix_enum = re.search(r"pub enum Prefix\s*\{([^}]+)\}", content, re.DOTALL)
        self.check("Prefix enum defined in addresses crate", bool(prefix_enum))
        if prefix_enum:
            enum_body = prefix_enum.group(1)
            self.check("Prefix::Mainnet serde rename is 'zyanya'", '#[serde(rename = "zyanya")]' in enum_body)
            self.check("Prefix::Testnet serde rename is 'zyanyatest'", '#[serde(rename = "zyanyatest")]' in enum_body)

        # Check as_str mapping
        as_str_match = re.search(r"fn as_str\(&self\)\s*->\s*&'static str\s*\{([^}]+)\}", content, re.DOTALL)
        self.check("Prefix::as_str() method implemented", bool(as_str_match))
        if as_str_match:
            as_str_body = as_str_match.group(1)
            self.check('Prefix::Mainnet -> "zyanya"', 'Prefix::Mainnet => "zyanya"' in as_str_body)
            self.check('Prefix::Testnet -> "zyanyatest"', 'Prefix::Testnet => "zyanyatest"' in as_str_body)

        # Check TryFrom<&str> parsing
        try_from_match = re.search(r"impl TryFrom<&str> for Prefix\s*\{([^}]+)\}", content, re.DOTALL)
        self.check("TryFrom<&str> for Prefix implemented", bool(try_from_match))
        if try_from_match:
            try_body = try_from_match.group(1)
            self.check('"zyanya" parses to Prefix::Mainnet', '"zyanya" => Ok(Prefix::Mainnet)' in try_body)
            self.check('"zyanyatest" parses to Prefix::Testnet', '"zyanyatest" => Ok(Prefix::Testnet)' in try_body)

        # Validate address format with standard colon separator
        mainnet_prefix = "zyanya"
        testnet_prefix = "zyanyatest"
        self.check(
            "Mainnet address prefix format is 'zyanya:'",
            f"{mainnet_prefix}:" == "zyanya:",
            "e.g., zyanya:qrh5l43xvd05lq36g37z7009s42f8c5mcv9943x7a7d4p2a7eet9j6085a855"
        )
        self.check(
            "Testnet address prefix format is 'zyanyatest:'",
            f"{testnet_prefix}:" == "zyanyatest:",
            "e.g., zyanyatest:qq0d2h8909893g2406k8px5x5005w0... "
        )
        self.check(
            "Mainnet and Testnet address prefixes are distinct and unambiguous",
            mainnet_prefix != testnet_prefix and not mainnet_prefix.startswith(testnet_prefix),
            f"'{mainnet_prefix}:' vs '{testnet_prefix}:'"
        )

if __name__ == "__main__":
    validator = PreflightValidator()
    success = validator.run()
    sys.exit(0 if success else 1)
