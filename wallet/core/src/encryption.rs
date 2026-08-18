//!
//! Wallet data encryption module.
//!

use crate::imports::*;
use crate::result::Result;
use argon2::Argon2;
use chacha20poly1305::{
    aead::{AeadCore, AeadInPlace, KeyInit, OsRng},
    Key, XChaCha20Poly1305,
};
use rand::RngCore;
use sha2::{Digest, Sha256};
use std::ops::{Deref, DerefMut};
use zeroize::{Zeroize, ZeroizeOnDrop};

/// Encryption algorithms supported by the Wallet framework.
#[derive(Default, Clone, Copy, Debug, Serialize, Deserialize, BorshSerialize, BorshDeserialize)]
pub enum EncryptionKind {
    #[default]
    XChaCha20Poly1305,
}

/// Abstract data container that can contain either plain or encrypted data and
/// transform the data between the two states.
#[derive(Clone, Debug, Serialize, Deserialize, BorshSerialize, BorshDeserialize)]
#[serde(tag = "encryptable", content = "payload")]
pub enum Encryptable<T> {
    #[serde(rename = "plain")]
    Plain(T),
    #[serde(rename = "xchacha20poly1305")]
    XChaCha20Poly1305(Encrypted),
}

impl<T> Zeroize for Encryptable<T>
where
    T: Zeroize,
{
    fn zeroize(&mut self) {
        match self {
            Self::Plain(t) => t.zeroize(),
            Self::XChaCha20Poly1305(e) => e.zeroize(),
        }
    }
}

impl<T> Encryptable<T>
where
    T: Clone + Zeroize + BorshDeserialize + BorshSerialize,
{
    pub fn is_encrypted(&self) -> bool {
        !matches!(self, Self::Plain(_))
    }

    pub fn decrypt(&self, secret: Option<&Secret>) -> Result<Decrypted<T>> {
        match self {
            Self::Plain(v) => Ok(Decrypted::new(v.clone())),
            Self::XChaCha20Poly1305(v) => {
                if let Some(secret) = secret {
                    Ok(v.decrypt(secret)?)
                } else {
                    Err("Decryption secret is 'None' when the data is encrypted!".into())
                }
            }
        }
    }

    pub fn encrypt(&self, secret: &Secret, encryption_kind: EncryptionKind) -> Result<Encrypted> {
        match self {
            Self::Plain(v) => Ok(Decrypted::new(v.clone()).encrypt(secret, encryption_kind)?),
            Self::XChaCha20Poly1305(v) => match encryption_kind {
                EncryptionKind::XChaCha20Poly1305 => Ok(v.clone()),
            },
        }
    }

    pub fn into_encrypted(&self, secret: &Secret, encryption_kind: EncryptionKind) -> Result<Self> {
        match self {
            Self::Plain(v) => Ok(Self::XChaCha20Poly1305(Decrypted::new(v.clone()).encrypt(secret, encryption_kind)?)),
            Self::XChaCha20Poly1305(v) => Ok(Self::XChaCha20Poly1305(v.clone())),
        }
    }

    pub fn into_decrypted(self, secret: &Secret) -> Result<Self> {
        match self {
            Self::Plain(v) => Ok(Self::Plain(v)),
            Self::XChaCha20Poly1305(v) => Ok(Self::Plain(v.decrypt::<T>(secret)?.unwrap())),
        }
    }
}

impl<T> From<T> for Encryptable<T> {
    fn from(value: T) -> Self {
        Encryptable::Plain(value)
    }
}

/// Abstract decrypted data container.
#[derive(Clone, BorshSerialize, BorshDeserialize)]
pub struct Decrypted<T>(pub(crate) T)
where
    T: BorshSerialize + BorshDeserialize;

impl<T> std::fmt::Debug for Decrypted<T>
where
    T: BorshSerialize + BorshDeserialize,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_tuple("Decrypted").field(&"[REDACTED]").finish()
    }
}

impl<T> Zeroize for Decrypted<T>
where
    T: BorshSerialize + BorshDeserialize + Zeroize,
{
    fn zeroize(&mut self) {
        self.0.zeroize();
    }
}

impl<T> ZeroizeOnDrop for Decrypted<T> where T: BorshSerialize + BorshDeserialize + Zeroize {}

impl<T> AsRef<T> for Decrypted<T>
where
    T: BorshSerialize + BorshDeserialize,
{
    fn as_ref(&self) -> &T {
        &self.0
    }
}

impl<T> Deref for Decrypted<T>
where
    T: BorshSerialize + BorshDeserialize,
{
    type Target = T;
    fn deref(&self) -> &T {
        &self.0
    }
}

impl<T> DerefMut for Decrypted<T>
where
    T: BorshSerialize + BorshDeserialize,
{
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl<T> AsMut<T> for Decrypted<T>
where
    T: BorshSerialize + BorshDeserialize,
{
    fn as_mut(&mut self) -> &mut T {
        &mut self.0
    }
}

impl<T> Decrypted<T>
where
    T: BorshSerialize + BorshDeserialize,
{
    pub fn new(value: T) -> Self {
        Self(value)
    }

    pub fn encrypt(&self, secret: &Secret, encryption_kind: EncryptionKind) -> Result<Encrypted> {
        let bytes = borsh::to_vec(&self.0)?;
        let encrypted = match encryption_kind {
            EncryptionKind::XChaCha20Poly1305 => encrypt_xchacha20poly1305(bytes.as_slice(), secret)?,
        };
        Ok(Encrypted::new(encryption_kind, encrypted))
    }

    pub fn unwrap(self) -> T {
        self.0
    }
}

/// Encrypted data container (wraps an encrypted payload)
#[derive(Clone, Serialize, Deserialize, BorshSerialize, BorshDeserialize)]
pub struct Encrypted {
    encryption_kind: EncryptionKind,
    payload: Vec<u8>,
}

impl Zeroize for Encrypted {
    fn zeroize(&mut self) {
        self.payload.zeroize();
    }
}

impl std::fmt::Debug for Encrypted {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Encrypted").field("encryption_kind", &self.encryption_kind).field("payload", &"[ENCRYPTED]").finish()
    }
}

impl Encrypted {
    pub fn new(encryption_kind: EncryptionKind, payload: Vec<u8>) -> Self {
        Encrypted { encryption_kind, payload }
    }

    pub fn replace(&mut self, from: Encrypted) {
        self.payload = from.payload;
    }

    pub fn kind(&self) -> EncryptionKind {
        self.encryption_kind
    }

    pub fn decrypt<T>(&self, secret: &Secret) -> Result<Decrypted<T>>
    where
        T: BorshSerialize + BorshDeserialize,
    {
        match self.encryption_kind {
            EncryptionKind::XChaCha20Poly1305 => {
                let decrypted = decrypt_xchacha20poly1305(&self.payload, secret)?;
                Ok(Decrypted(T::try_from_slice(decrypted.as_ref())?))
            }
        }
    }
}

/// Produces `SHA256` hash of the given data.
#[inline]
pub fn sha256_hash(data: &[u8]) -> Secret {
    let mut sha256 = Sha256::default();
    sha256.update(data);
    Secret::new(sha256.finalize().to_vec())
}

/// Produces `SHA256d` hash of the given data.
#[inline]
pub fn sha256d_hash(data: &[u8]) -> Secret {
    let mut sha256 = Sha256::default();
    sha256.update(data);
    sha256_hash(sha256.finalize().as_slice())
}

/// Produces `argon2sha256iv` hash of the given data.
///
/// This is a *deterministic* password-hash helper that derives the Argon2 salt from
/// `sha256(data)`. It is suitable for password verification (the WASM
/// `js_argon2_sha256iv_phash_*` API) but MUST NOT be used as the encryption KDF: a
/// deterministic salt means the same password always yields the same key, defeating
/// semantic security. Use [`argon2_hash`] with a fresh random salt for encryption.
pub fn argon2_sha256iv_hash(data: &[u8], byte_length: usize) -> Result<Secret> {
    let salt = sha256_hash(data);
    let mut key = vec![0u8; byte_length];
    Argon2::default().hash_password_into(data, salt.as_ref(), &mut key)?;
    Ok(key.into())
}

/// Derive a key from `data` and an explicit `salt` using Argon2.
///
/// Unlike [`argon2_sha256iv_hash`], the salt is caller-supplied (typically random),
/// so the same password produces a different key each time. This is the correct KDF
/// to use for encryption.
pub fn argon2_hash(data: &[u8], salt: &[u8], byte_length: usize) -> Result<Secret> {
    let mut key = vec![0u8; byte_length];
    Argon2::default().hash_password_into(data, salt, &mut key)?;
    Ok(key.into())
}

/// Encrypts the given data using `XChaCha20Poly1305` algorithm.
///
/// Payload layout: `salt(16) || nonce(24) || ciphertext+tag`.
/// A fresh 16-byte random salt is generated per encryption so that the same plaintext
/// and password never yield identical ciphertext (F-M-22).
pub fn encrypt_xchacha20poly1305(data: &[u8], secret: &Secret) -> Result<Vec<u8>> {
    // F-M-22: use a random salt instead of sha256(secret) so the derived key is unique
    // per encryption even for an identical password.
    let mut salt = [0u8; 16];
    OsRng.fill_bytes(&mut salt);
    let private_key_bytes = argon2_hash(secret.as_ref(), &salt, 32)?;
    let key = Key::from_slice(private_key_bytes.as_ref());
    let cipher = XChaCha20Poly1305::new(key);
    let nonce = XChaCha20Poly1305::generate_nonce(&mut OsRng); // 192-bits; unique per message
    let mut buffer = data.to_vec();
    buffer.reserve(16);
    cipher.encrypt_in_place(&nonce, &[], &mut buffer)?;
    // Layout: salt(16) || nonce(24) || ciphertext+tag
    buffer.splice(0..0, nonce.iter().cloned());
    buffer.splice(0..0, salt.iter().cloned());
    Ok(buffer)
}

/// Decrypts the given data using `XChaCha20Poly1305` algorithm.
///
/// Expected payload layout: `salt(16) || nonce(24) || ciphertext+tag` (F-M-22).
/// Returns an error if the ciphertext is too short to contain the salt, nonce, and
/// minimum AEAD tag (F-M-10).
pub fn decrypt_xchacha20poly1305(data: &[u8], secret: &Secret) -> Result<Secret> {
    // F-M-10: reject short ciphertexts before slicing instead of panicking.
    // Minimum: salt(16) + nonce(24) + AEAD tag(16) = 56 bytes.
    const MIN_LEN: usize = 16 + 24 + 16;
    if data.len() < MIN_LEN {
        return Err(format!("ciphertext too short: {} bytes (minimum {})", data.len(), MIN_LEN).into());
    }
    // F-M-22: read the random salt prefix and derive the key with it.
    let (salt, rest) = data.split_at(16);
    let (nonce, ciphertext) = rest.split_at(24);
    let private_key_bytes = argon2_hash(secret.as_ref(), salt, 32)?;
    let key = Key::from_slice(private_key_bytes.as_ref());
    let cipher = XChaCha20Poly1305::new(key);
    let mut buffer = ciphertext.to_vec();
    cipher.decrypt_in_place(nonce.into(), &[], &mut buffer)?;
    Ok(Secret::new(buffer))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_wallet_argon2() {
        println!("testing argon2 hash");
        let password = b"user_password";
        let hash = argon2_sha256iv_hash(password, 32).unwrap();
        let hash_hex = hash.as_ref().to_hex();
        // println!("argon2hash: {:?}", hash_hex);
        assert_eq!(hash_hex, "a79b661f0defd1960a4770889e19da0ce2fde1e98ca040f84ab9b2519ca46234");
    }

    #[test]
    fn test_wallet_encrypt_decrypt() -> Result<()> {
        println!("testing encrypt/decrypt");

        let password = b"password";
        let original = b"hello world".to_vec();
        // println!("original: {}", original.to_hex());
        let password = Secret::new(password.to_vec());
        let encrypted = encrypt_xchacha20poly1305(&original, &password).unwrap();
        // println!("encrypted: {}", encrypted.to_hex());
        let decrypted = decrypt_xchacha20poly1305(&encrypted, &password).unwrap();
        // println!("decrypted: {}", decrypted.to_hex());
        assert_eq!(decrypted.as_ref(), original);

        Ok(())
    }

    #[test]
    fn test_wallet_encrypt_salt_uniqueness() {
        // F-M-22: two encryptions of the same plaintext + password must yield
        // different ciphertexts because a fresh random salt is used each time.
        let password = Secret::new(b"password".to_vec());
        let original = b"hello world".to_vec();
        let a = encrypt_xchacha20poly1305(&original, &password).unwrap();
        let b = encrypt_xchacha20poly1305(&original, &password).unwrap();
        assert_ne!(a, b, "identical plaintext/password must not produce identical ciphertext");
    }

    #[test]
    fn test_wallet_decrypt_short_ciphertext_errors() {
        // F-M-10: short ciphertext must return an error, not panic.
        let password = Secret::new(b"password".to_vec());
        let short = vec![0u8; 10];
        let result = decrypt_xchacha20poly1305(&short, &password);
        assert!(result.is_err(), "short ciphertext should return an error");
    }
}
