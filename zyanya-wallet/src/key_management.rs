use std::fs;
use std::path::{Path, PathBuf};
use secp256k1::{Keypair, PublicKey, Secp256k1, SecretKey, XOnlyPublicKey};
use rand::rngs::OsRng;
use zyanya_addresses::{Address, Prefix, Version};
use zyanya_consensus_core::{
    sign::{sign, verify},
    tx::{MutableTransaction, Transaction, UtxoEntry},
};
use thiserror::Error;

#[derive(Error, Debug)]
pub enum KeyManagementError {
    #[error("Failed to parse secret key from hex: {0}")]
    InvalidHex(String),
    #[error("Secp256k1 error: {0}")]
    Secp256k1(#[from] secp256k1::Error),
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    #[error("Transaction signing failed: {0}")]
    SignError(String),
}

#[derive(Clone)]
pub struct WalletKeypair {
    pub secret_key: SecretKey,
    #[allow(dead_code)]
    pub public_key: PublicKey,
    pub xonly_pubkey: XOnlyPublicKey,
    pub address: Address,
}

impl std::fmt::Debug for WalletKeypair {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("WalletKeypair")
            .field("address", &self.address.to_string())
            .finish()
    }
}

impl WalletKeypair {
    /// Generate a brand new random keypair with Prefix::Devnet ("zyanyadev:...")
    pub fn generate() -> Self {
        let secp = Secp256k1::new();
        let mut rng = OsRng;
        let (secret_key, public_key) = secp.generate_keypair(&mut rng);
        let (xonly_pubkey, _) = public_key.x_only_public_key();
        let payload = xonly_pubkey.serialize();
        let address = Address::new(Prefix::Devnet, Version::PubKey, &payload);

        Self {
            secret_key,
            public_key,
            xonly_pubkey,
            address,
        }
    }

    /// Load a keypair from a 64-character hex secret key string
    pub fn from_secret_hex(hex_str: &str) -> Result<Self, KeyManagementError> {
        let clean = hex_str.trim().trim_start_matches("0x");
        let mut bytes = [0u8; 32];
        faster_hex::hex_decode(clean.as_bytes(), &mut bytes)
            .map_err(|e| KeyManagementError::InvalidHex(e.to_string()))?;

        let secret_key = SecretKey::from_slice(&bytes)?;
        let secp = Secp256k1::new();
        let public_key = PublicKey::from_secret_key(&secp, &secret_key);
        let (xonly_pubkey, _) = public_key.x_only_public_key();
        let payload = xonly_pubkey.serialize();
        let address = Address::new(Prefix::Devnet, Version::PubKey, &payload);

        Ok(Self {
            secret_key,
            public_key,
            xonly_pubkey,
            address,
        })
    }

    /// Get secret key in hex format
    pub fn secret_hex(&self) -> String {
        self.secret_key.display_secret().to_string()
    }

    /// Default file path for wallet key (~/.zyanya/wallet.key)
    pub fn default_key_path() -> PathBuf {
        if let Some(home) = dirs::home_dir() {
            home.join(".zyanya").join("wallet.key")
        } else {
            PathBuf::from("wallet.key")
        }
    }

    /// Save private key hex to file
    pub fn save_to_file(&self, path: &Path) -> Result<(), KeyManagementError> {
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)?;
        }
        let content = format!("{}\n", self.secret_hex());
        fs::write(path, content)?;
        Ok(())
    }

    /// Load private key from file
    pub fn load_from_file(path: &Path) -> Result<Self, KeyManagementError> {
        let content = fs::read_to_string(path)?;
        Self::from_secret_hex(&content)
    }

    /// Sign a transaction using this wallet's private key (Schnorr signature)
    pub fn sign_transaction(
        &self,
        unsigned_tx: Transaction,
        utxos: Vec<UtxoEntry>,
    ) -> Result<Transaction, KeyManagementError> {
        let keypair = Keypair::from_secret_key(secp256k1::SECP256K1, &self.secret_key);
        let signable = MutableTransaction::with_entries(unsigned_tx, utxos);
        let signed = sign(signable, keypair);

        if let Err(e) = verify(&signed.as_verifiable()) {
            return Err(KeyManagementError::SignError(e.to_string()));
        }

        Ok(signed.tx)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_key_generation_and_serialization() {
        let wallet = WalletKeypair::generate();
        assert!(wallet.address.to_string().starts_with("zyanyadev:"));
        let hex = wallet.secret_hex();
        assert_eq!(hex.len(), 64);

        let restored = WalletKeypair::from_secret_hex(&hex).unwrap();
        assert_eq!(wallet.address.to_string(), restored.address.to_string());
    }

    #[test]
    fn test_save_and_load_file() {
        let wallet = WalletKeypair::generate();
        let temp_dir = std::env::temp_dir();
        let file_path = temp_dir.join("test_zyanya_wallet.key");

        wallet.save_to_file(&file_path).unwrap();
        let loaded = WalletKeypair::load_from_file(&file_path).unwrap();

        assert_eq!(wallet.address.to_string(), loaded.address.to_string());
        assert_eq!(wallet.secret_hex(), loaded.secret_hex());

        let _ = fs::remove_file(file_path);
    }
}
