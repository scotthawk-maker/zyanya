use crate::{block::Block, header::Header, subnets::SUBNETWORK_ID_COINBASE, tx::Transaction};
use zyanya_hashes::{Hash, ZERO_HASH};
use zyanya_muhash::EMPTY_MUHASH;

/// The constants uniquely representing the genesis block
#[derive(Clone, Debug)]
pub struct GenesisBlock {
    pub hash: Hash,
    pub version: u16,
    pub hash_merkle_root: Hash,
    pub utxo_commitment: Hash,
    pub timestamp: u64,
    pub bits: u32,
    pub nonce: u64,
    pub daa_score: u64,
    pub coinbase_payload: &'static [u8],
}

impl GenesisBlock {
    pub fn build_genesis_transactions(&self) -> Vec<Transaction> {
        vec![Transaction::new(0, Vec::new(), Vec::new(), 0, SUBNETWORK_ID_COINBASE, 0, self.coinbase_payload.to_vec())]
    }
}

impl From<&GenesisBlock> for Header {
    fn from(genesis: &GenesisBlock) -> Self {
        Header::new_finalized(
            genesis.version,
            Vec::new(),
            genesis.hash_merkle_root,
            ZERO_HASH,
            genesis.utxo_commitment,
            genesis.timestamp,
            genesis.bits,
            genesis.nonce,
            genesis.daa_score,
            0.into(),
            0,
            ZERO_HASH,
        )
    }
}

impl From<&GenesisBlock> for Block {
    fn from(genesis: &GenesisBlock) -> Self {
        Block::new(genesis.into(), genesis.build_genesis_transactions())
    }
}

impl From<(&Header, &'static [u8])> for GenesisBlock {
    fn from((header, payload): (&Header, &'static [u8])) -> Self {
        Self {
            hash: header.hash,
            version: header.version,
            hash_merkle_root: header.hash_merkle_root,
            utxo_commitment: header.utxo_commitment,
            timestamp: header.timestamp,
            bits: header.bits,
            nonce: header.nonce,
            daa_score: header.daa_score,
            coinbase_payload: payload,
        }
    }
}

/// The genesis block of the block-DAG which serves as the public transaction ledger for the main network.
pub const MAINNET_GENESIS: GenesisBlock = GenesisBlock {
    hash: Hash::from_bytes([
        0x99, 0xF5, 0xE7, 0xF1, 0xE1, 0xF3, 0x2E, 0xFD, 0xBA, 0x33, 0xA5, 0x5A, 0x6B, 0xCD, 0x18, 0x6B,
        0xAE, 0xE9, 0x82, 0x7A, 0x63, 0xC8, 0xDD, 0x17, 0x3C, 0xB2, 0x72, 0x13, 0x67, 0xC5, 0x78, 0x15,
    ]),
    version: 0,
    hash_merkle_root: Hash::from_bytes([
        0xE7, 0x7C, 0x8E, 0x7C, 0xBE, 0x98, 0xE9, 0xB0, 0x48, 0xF8, 0x81, 0x63, 0x81, 0x96, 0x38, 0x67,
        0x3A, 0xD7, 0xCC, 0xDF, 0xE5, 0xCF, 0xBB, 0x98, 0x78, 0x0A, 0x2D, 0x1A, 0x51, 0xA6, 0xF5, 0xCE,
    ]),
    utxo_commitment: EMPTY_MUHASH,
    timestamp: 1770000000000,
    bits: 536999497, // Prime number
    nonce: 287000,   // Custom mainnet genesis nonce
    daa_score: 0,    // Checkpoint DAA score
    #[rustfmt::skip]
    coinbase_payload: &[
        0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, // Blue score
        0x00, 0xF4, 0x05, 0x2A, 0x01, 0x00, 0x00, 0x00, // Subsidy = 5_000_000_000 sompi (50 ZYAN)
        0x00, 0x00,                                     // Script version
        0x01,                                           // Varint
        0x00,                                           // OP-FALSE
        0x5A, 0x59, 0x41, 0x4E, 0x2D, 0x4D, 0x41, 0x49, // ZYAN-MAINNET magic bytes payload
        0x4E, 0x4E, 0x45, 0x54,
    ],
};

pub const GENESIS: GenesisBlock = MAINNET_GENESIS;

pub const TESTNET_GENESIS: GenesisBlock = GenesisBlock {
    hash: Hash::from_bytes([
        0xC6, 0x86, 0x6E, 0x0B, 0xA3, 0x07, 0x59, 0x96, 0xF0, 0x66, 0x9B, 0xD7, 0xBD, 0xC2, 0x6C, 0x00,
        0x08, 0x6D, 0x8C, 0xD3, 0x45, 0xDD, 0xA6, 0xF1, 0xBE, 0x04, 0xF4, 0x03, 0xAF, 0x25, 0xCF, 0x6A,
    ]),
    version: 0,
    hash_merkle_root: Hash::from_bytes([
        0xCD, 0x03, 0x30, 0x09, 0x64, 0xE7, 0xA6, 0x55, 0xCE, 0x2D, 0x57, 0x09, 0xA5, 0xA5, 0xD7, 0x3C,
        0xC1, 0x79, 0xF6, 0x68, 0x13, 0xFF, 0xB1, 0x40, 0x2B, 0x3F, 0x03, 0x2F, 0x75, 0xA8, 0x06, 0x8E,
    ]),
    utxo_commitment: EMPTY_MUHASH,
    timestamp: 1770000000000,
    bits: 511699987, // Prime number
    nonce: 182100,   // Custom testnet genesis nonce
    daa_score: 0,
    #[rustfmt::skip]
    coinbase_payload: &[
        0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, // Blue score
        0x00, 0xF4, 0x05, 0x2A, 0x01, 0x00, 0x00, 0x00, // Subsidy = 5_000_000_000 sompi (50 ZYAN)
        0x00, 0x00,                                     // Script version
        0x01,                                           // Varint
        0x00,                                           // OP-FALSE
        0x5A, 0x59, 0x4E, 0x54, 0x2D, 0x54, 0x45, 0x53, // ZYNT-TESTNET magic bytes payload
        0x54, 0x4E, 0x45, 0x54,
    ],
};

pub const TESTNET11_GENESIS: GenesisBlock = GenesisBlock {
    hash: Hash::from_bytes([
        0xAD, 0x64, 0xC6, 0x3F, 0x4B, 0xD6, 0xA9, 0x36, 0xA5, 0x2D, 0xE3, 0xFD, 0x26, 0x94, 0x74, 0x9D, 0x77, 0xFE, 0x7B, 0xD5, 0x96,
        0xE8, 0x46, 0xD8, 0x26, 0x90, 0xB7, 0xB4, 0xD7, 0xF5, 0x3C, 0x8F,
    ]),
    version: 0,
    hash_merkle_root: Hash::from_bytes([
        0xD4, 0x08, 0xA5, 0xD2, 0xF6, 0x40, 0xC2, 0x75, 0x7D, 0x69, 0x84, 0x22, 0xF5, 0xEF, 0xFB, 0xD5, 0xF3, 0x9B, 0xA8, 0x79, 0x9D,
        0x2C, 0x1C, 0x8E, 0x74, 0xAA, 0x2B, 0x4D, 0xA4, 0x2E, 0xE0, 0x77,
    ]),
    utxo_commitment: EMPTY_MUHASH,
    timestamp: 1713884672545,
    bits: 504154830, // see `gen_testnet11_genesis`
    nonce: 314159,
    daa_score: 0,
    #[rustfmt::skip]
    coinbase_payload: &[
        0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, // Blue score
        0x00, 0xE1, 0xF5, 0x05, 0x00, 0x00, 0x00, 0x00, // Subsidy
        0x00, 0x00,                                     // Script version
        0x01,                                           // Varint
        0x00,                                           // OP-FALSE
        0x6B, 0x61, 0x73, 0x70, 0x61, 0x2D, 0x74, 0x65,
    ],
};

pub const SIMNET_GENESIS: GenesisBlock = GenesisBlock {
    hash: Hash::from_bytes([
        0x56, 0xBB, 0x87, 0xCF, 0x18, 0x77, 0x7B, 0x76, 0x35, 0x8E, 0xEE, 0xF0, 0x20, 0xA9, 0x01, 0xCD, 0xDD, 0xDC, 0x0B, 0xA4, 0x46,
        0xC0, 0x99, 0x2D, 0xE2, 0x7C, 0xC2, 0xA8, 0x9E, 0xC7, 0xA1, 0x30,
    ]),
    version: 0,
    hash_merkle_root: Hash::from_bytes([
        0x85, 0x81, 0x84, 0xD0, 0x98, 0x16, 0x40, 0x4F, 0xD7, 0xD7, 0x96, 0xFB, 0xDE, 0x60, 0xAC, 0x4B, 0x99, 0x29, 0xB9, 0x18, 0x63,
        0x39, 0xDA, 0x23, 0x08, 0x3C, 0xDF, 0xC3, 0x5F, 0x13, 0x8F, 0xC6,
    ]),
    utxo_commitment: EMPTY_MUHASH,
    timestamp: 1713885012324,
    bits: 543656363, // Prime number
    nonce: 2,        // Two
    daa_score: 0,
    #[rustfmt::skip]
    coinbase_payload: &[
        0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, // Blue score
        0x00, 0xE1, 0xF5, 0x05, 0x00, 0x00, 0x00, 0x00, // Subsidy
        0x00, 0x00,                                     // Script version
        0x01,                                           // Varint
        0x00,                                           // OP-FALSE
        0x54, 0x36, 0x56, 0x36, 0x56, 0x91, 0x80, 0x90, // Euler's number * 2 = 5.436563656918090
    ],
};

pub const DEVNET_GENESIS: GenesisBlock = GenesisBlock {
    hash: Hash::from_bytes([
        0x6C, 0x34, 0x89, 0xBF, 0xB5, 0x92, 0xCA, 0x0A, 0x0C, 0x12, 0xED, 0xB7, 0xAD, 0x86, 0x2D, 0x62, 0x27, 0x92, 0x3E, 0xC2, 0xD2,
        0x77, 0x7E, 0x0D, 0xFD, 0x93, 0xF3, 0xC5, 0xB8, 0xA5, 0x5C, 0x35,
    ]),
    version: 0,
    hash_merkle_root: Hash::from_bytes([
        0x45, 0x7F, 0x6D, 0xF5, 0x76, 0x25, 0xCF, 0xC9, 0x4A, 0x63, 0x16, 0x9E, 0xBA, 0xC8, 0xE1, 0x86, 0xCF, 0x1B, 0x5F, 0x1E, 0xF6,
        0x8D, 0x1A, 0xEF, 0x3B, 0x8D, 0x3F, 0xFC, 0xC2, 0x6C, 0x01, 0xE4,
    ]),
    utxo_commitment: EMPTY_MUHASH,
    timestamp: 1713884849877,
    bits: 541034453, // Prime number
    nonce: 241421,   // Silver ratio
    daa_score: 0,
    #[rustfmt::skip]
    coinbase_payload: &[
        0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, // Blue score
        0x00, 0xE1, 0xF5, 0x05, 0x00, 0x00, 0x00, 0x00, // Subsidy
        0x00, 0x00,                                     // Script version
        0x01,                                           // Varint
        0x00,                                           // OP-FALSE
        0x24, 0x14, 0x21, 0x35, 0x62, 0x37, 0x30, 0x95, // Silver ratio
    ],
};

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{config::bps::Testnet11Bps, merkle::calc_hash_merkle_root};

    #[test]
    fn test_genesis_hashes() {
        [MAINNET_GENESIS, GENESIS, TESTNET_GENESIS, TESTNET11_GENESIS, SIMNET_GENESIS, DEVNET_GENESIS].into_iter().for_each(|genesis| {
            let block: Block = (&genesis).into();
            assert_hashes_eq(calc_hash_merkle_root(block.transactions.iter(), false), block.header.hash_merkle_root);
            assert_hashes_eq(block.hash(), genesis.hash);
        });
    }

    #[test]
    fn gen_testnet11_genesis() {
        let bps = Testnet11Bps::bps();
        let mut genesis = TESTNET_GENESIS;
        let target = zyanya_math::Uint256::from_compact_target_bits(genesis.bits);
        let scaled_target = target * bps / 100;
        let scaled_bits = scaled_target.compact_target_bits();
        genesis.bits = scaled_bits;
        if genesis.bits != TESTNET11_GENESIS.bits {
            panic!("Testnet 11: new bits: {}\nnew hash: {:#04x?}", scaled_bits, Block::from(&genesis).hash().as_bytes());
        }
    }

    fn assert_hashes_eq(got: Hash, expected: Hash) {
        if got != expected {
            // Special hex print to ease changing the genesis hash according to the print if needed
            panic!("Got hash {:#04x?} while expecting {:#04x?}", got.as_bytes(), expected.as_bytes());
        }
    }
}
