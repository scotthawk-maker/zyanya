//!
//! Secret container for sensitive data. Performs data erasure (zeroization) on drop.
//!

use crate::imports::*;

/// Secret container for sensitive data. Performs memory erasure (zeroization) on drop.
///
/// `Clone` is implemented manually rather than derived so that the duplication is an
/// explicit, audited operation. A `Clone` impl cannot zeroize its `&self` source; the
/// mitigation is that *every* `Secret` (including clones) is zeroized on drop by the
/// `Drop` impl below. Callers should prefer borrowing (`&Secret`) over cloning where
/// possible to avoid creating additional copies of secret material.
///
/// NOTE: `Serialize`/`Deserialize`/`Borsh*` are retained because API message structs in
/// `wallet/core/src/api/message.rs` derive them and require `Secret` to be
/// (de)serializable. Plaintext serialization of secret material is a residual risk that
/// should be addressed in a follow-up (out of scope for this batch).
#[derive(Serialize, Deserialize, BorshSerialize, BorshDeserialize)]
pub struct Secret(Vec<u8>);

impl Clone for Secret {
    /// Explicit, audited duplication of secret material.
    ///
    /// Cloning creates a second copy of the secret bytes. The original `&self` cannot
    /// be zeroized by this impl (it is borrowed), but every `Secret` — original or
    /// clone — is zeroized on drop via the `Drop` impl, so no copy persists in memory
    /// beyond its lifetime. Prefer `&Secret` borrows over cloning to minimize the
    /// number of in-memory copies.
    fn clone(&self) -> Self {
        Secret(self.0.clone())
    }
}

impl Secret {
    pub fn new(data: Vec<u8>) -> Self {
        Self(data)
    }

    pub fn as_str(&self) -> Result<&str> {
        Ok(std::str::from_utf8(&self.0)?)
    }
}

impl AsRef<[u8]> for Secret {
    fn as_ref(&self) -> &[u8] {
        &self.0
    }
}

impl From<Vec<u8>> for Secret {
    fn from(vec: Vec<u8>) -> Self {
        Secret(vec)
    }
}

impl From<&[u8]> for Secret {
    fn from(slice: &[u8]) -> Self {
        Secret(slice.to_vec())
    }
}

impl From<&str> for Secret {
    fn from(s: &str) -> Self {
        Secret(s.trim().as_bytes().to_vec())
    }
}

impl From<String> for Secret {
    fn from(mut s: String) -> Self {
        let secret = Secret(s.trim().as_bytes().to_vec());
        s.zeroize();
        secret
    }
}

impl Zeroize for Secret {
    fn zeroize(&mut self) {
        self.0.zeroize()
    }
}

impl Drop for Secret {
    fn drop(&mut self) {
        self.zeroize()
    }
}

impl std::fmt::Debug for Secret {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Secret").field("secret", &"********").finish()
    }
}
