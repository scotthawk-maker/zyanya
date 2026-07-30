use std::fs;
use std::path::{Path, PathBuf};
use secp256k1::{Keypair, PublicKey, Secp256k1, SecretKey, XOnlyPublicKey};
use rand::rngs::OsRng;
use zyanya_addresses::{Address, Prefix, Version};
use zyanya_consensus_core::{
    sign::{sign, verify},
    tx::{MutableTransaction, Transaction, UtxoEntry},
};
use zyanya_bip32::{ExtendedPrivateKey, Language, Mnemonic, WordCount};
use thiserror::Error;

#[derive(Error, Debug)]
pub enum KeyManagementError {
    #[error("Failed to parse secret key from hex: {0}")]
    InvalidHex(String),
    #[error("BIP-39 Mnemonic error: {0}")]
    InvalidMnemonic(String),
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
    /// Construct WalletKeypair from a secp256k1 SecretKey
    pub fn from_secret_key(secret_key: SecretKey, prefix: Prefix) -> Self {
        let secp = Secp256k1::new();
        let public_key = PublicKey::from_secret_key(&secp, &secret_key);
        let (xonly_pubkey, _) = public_key.x_only_public_key();
        let payload = xonly_pubkey.serialize();
        let address = Address::new(prefix, Version::PubKey, &payload);

        Self {
            secret_key,
            public_key,
            xonly_pubkey,
            address,
        }
    }

    /// Generate a brand new random keypair with the given prefix
    pub fn generate(prefix: Prefix) -> Self {
        let secp = Secp256k1::new();
        let mut rng = OsRng;
        let (secret_key, _) = secp.generate_keypair(&mut rng);
        Self::from_secret_key(secret_key, prefix)
    }

    /// Generate a brand new 24-word BIP-39 mnemonic phrase and derive the secp256k1 keypair.
    /// Returns (WalletKeypair, 24_word_mnemonic_string).
    pub fn generate_mnemonic(passphrase: Option<&str>, prefix: Prefix) -> Result<(Self, String), KeyManagementError> {
        let mnemonic = Mnemonic::random(WordCount::Words24, Language::English)
            .map_err(|e| KeyManagementError::InvalidMnemonic(e.to_string()))?;
        let phrase = mnemonic.phrase().to_string();
        let seed = mnemonic.to_seed(passphrase.unwrap_or(""));
        let xprv = ExtendedPrivateKey::<SecretKey>::new(seed)
            .map_err(|e| KeyManagementError::InvalidMnemonic(e.to_string()))?;
        let secret_key = *xprv.private_key();
        Ok((Self::from_secret_key(secret_key, prefix), phrase))
    }

    /// Restore a keypair from a 24-word BIP-39 mnemonic phrase (+ optional passphrase).
    pub fn from_mnemonic(phrase: &str, passphrase: Option<&str>, prefix: Prefix) -> Result<Self, KeyManagementError> {
        let clean_phrase = phrase.split_whitespace().collect::<Vec<_>>().join(" ");
        let mnemonic = Mnemonic::new(&clean_phrase, Language::English)
            .map_err(|e| KeyManagementError::InvalidMnemonic(e.to_string()))?;
        let seed = mnemonic.to_seed(passphrase.unwrap_or(""));
        let xprv = ExtendedPrivateKey::<SecretKey>::new(seed)
            .map_err(|e| KeyManagementError::InvalidMnemonic(e.to_string()))?;
        let secret_key = *xprv.private_key();
        Ok(Self::from_secret_key(secret_key, prefix))
    }

    /// Load a keypair from a 64-character hex secret key string
    pub fn from_secret_hex(hex_str: &str, prefix: Prefix) -> Result<Self, KeyManagementError> {
        let clean = hex_str.trim().trim_start_matches("0x");
        let mut bytes = [0u8; 32];
        faster_hex::hex_decode(clean.as_bytes(), &mut bytes)
            .map_err(|e| KeyManagementError::InvalidHex(e.to_string()))?;

        let secret_key = SecretKey::from_slice(&bytes)?;
        Ok(Self::from_secret_key(secret_key, prefix))
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

        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let perms = fs::Permissions::from_mode(0o600);
            fs::set_permissions(path, perms)?;
        }

        Ok(())
    }

    /// Load private key from file
    pub fn load_from_file(path: &Path, prefix: Prefix) -> Result<Self, KeyManagementError> {
        let content = fs::read_to_string(path)?;
        Self::from_secret_hex(&content, prefix)
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

/// Display 24-word BIP-39 mnemonic phrase formatted and numbered 1-24 with standard warning
pub fn display_mnemonic(phrase: &str) {
    let words: Vec<&str> = phrase.split_whitespace().collect();
    println!("\n================================================================================");
    println!("                         BIP-39 MNEMONIC SEED PHRASE                            ");
    println!("================================================================================");
    for (i, word) in words.iter().enumerate() {
        print!("{:2}. {:<15} ", i + 1, word);
        if (i + 1) % 3 == 0 {
            println!();
        }
    }
    if words.len() % 3 != 0 {
        println!();
    }
    println!("--------------------------------------------------------------------------------");
    println!("Write down these 24 words. They are your wallet. Anyone with these words controls your ZYAN. Never share them.");
    println!("================================================================================\n");
}

/// Parse a decimal string representing ZYAN (e.g. "0.1", "10.5", "10") into integer sompi (8 decimal places).
pub fn parse_zyan_to_sompi(s: &str) -> Result<u64, String> {
    use zyanya_consensus_core::constants::SOMPI_PER_ZYANYA;
    let s = s.trim();
    if s.is_empty() {
        return Err("Empty amount".to_string());
    }
    let parts: Vec<&str> = s.split('.').collect();
    if parts.len() > 2 {
        return Err("Invalid decimal format".to_string());
    }

    let whole_str = parts[0];
    let whole: u64 = if whole_str.is_empty() {
        0
    } else {
        whole_str.parse::<u64>().map_err(|e| format!("Invalid whole part: {}", e))?
    };

    let mut sompi = whole.checked_mul(SOMPI_PER_ZYANYA).ok_or_else(|| "Amount too large".to_string())?;

    if parts.len() == 2 {
        let frac_str = parts[1];
        if !frac_str.is_empty() {
            if frac_str.len() > 8 {
                return Err("Too many decimal places (max 8)".to_string());
            }
            if !frac_str.chars().all(|c| c.is_ascii_digit()) {
                return Err("Invalid characters in fractional part".to_string());
            }
            let padded = format!("{:0<8}", frac_str);
            let frac_val: u64 = padded.parse().map_err(|e| format!("Invalid fraction part: {}", e))?;
            sompi = sompi.checked_add(frac_val).ok_or_else(|| "Amount too large".to_string())?;
        }
    }

    Ok(sompi)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_key_generation_and_serialization() {
        let wallet = WalletKeypair::generate(Prefix::Devnet);
        assert!(wallet.address.to_string().starts_with("zyanyadev:"));
        let hex = wallet.secret_hex();
        assert_eq!(hex.len(), 64);

        let restored = WalletKeypair::from_secret_hex(&hex, Prefix::Devnet).unwrap();
        assert_eq!(wallet.address.to_string(), restored.address.to_string());
    }

    #[test]
    fn test_bip39_mnemonic_generation_and_import() {
        // Test generating mnemonic
        let (wallet, phrase) = WalletKeypair::generate_mnemonic(None, Prefix::Devnet).unwrap();
        let words: Vec<&str> = phrase.split_whitespace().collect();
        assert_eq!(words.len(), 24, "Generated mnemonic must have 24 words");

        // Test restoring from mnemonic
        let restored = WalletKeypair::from_mnemonic(&phrase, None, Prefix::Devnet).unwrap();
        assert_eq!(wallet.address.to_string(), restored.address.to_string());
        assert_eq!(wallet.secret_hex(), restored.secret_hex());
    }

    #[test]
    fn test_bip39_passphrase_derivation() {
        let (wallet, phrase) = WalletKeypair::generate_mnemonic(Some("secret123"), Prefix::Devnet).unwrap();
        
        // Importing with matching passphrase gives same address
        let restored_same = WalletKeypair::from_mnemonic(&phrase, Some("secret123"), Prefix::Devnet).unwrap();
        assert_eq!(wallet.address.to_string(), restored_same.address.to_string());

        // Importing without passphrase or different passphrase gives different address
        let restored_different = WalletKeypair::from_mnemonic(&phrase, None, Prefix::Devnet).unwrap();
        assert_ne!(wallet.address.to_string(), restored_different.address.to_string());
    }

    #[test]
    fn test_save_and_load_file() {
        let (wallet, _phrase) = WalletKeypair::generate_mnemonic(None, Prefix::Devnet).unwrap();
        let temp_dir = std::env::temp_dir();
        let file_path = temp_dir.join("test_zyanya_wallet_mnemonic.key");

        wallet.save_to_file(&file_path).unwrap();
        let loaded = WalletKeypair::load_from_file(&file_path, Prefix::Devnet).unwrap();

        assert_eq!(wallet.address.to_string(), loaded.address.to_string());
        assert_eq!(wallet.secret_hex(), loaded.secret_hex());

        let _ = fs::remove_file(file_path);
    }
}
