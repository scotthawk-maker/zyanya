//!
//! # Scoped Agent Session Keys & Policy Engine (Track 9)
//!
//! Provides cryptographically scoped, ephemeral session certificates for autonomous AI
//! agents. Enforces strict spending limits, daily velocity caps, contract address
//! whitelisting, and expiration to prevent hot-wallet draining via prompt-injection attacks.
//!

use serde::{Deserialize, Serialize};
use thiserror::Error;
use zyanya_wallet_keys::secret::Secret;

/// Error variants emitted by the Session Policy Engine.
#[derive(Debug, Error, PartialEq, Eq, Clone, Serialize, Deserialize)]
pub enum SessionPolicyError {
    #[error("Session key has expired at timestamp {expires_at}, current time is {current_time}")]
    Expired { expires_at: u64, current_time: u64 },

    #[error("Transaction amount {requested} Sompi exceeds maximum allowed per-transaction spend limit of {limit} Sompi")]
    ExceedsPerTxLimit { limit: u64, requested: u64 },

    #[error("Attempted spend of {attempted_total} Sompi exceeds rolling 24-hour daily velocity limit of {limit} Sompi")]
    ExceedsDailyLimit { limit: u64, attempted_total: u64 },

    #[error("Target destination '{recipient}' is not in the authorized contract whitelist")]
    DestinationNotWhitelisted { recipient: String },

    #[error("Arithmetic overflow encountered during velocity calculation")]
    Overflow,

    #[error("Invalid session certificate signature")]
    InvalidSignature,
}

/// Spending boundaries and execution constraints for an agent session key.
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct SessionPolicy {
    /// Maximum Sompi permitted in a single transaction output.
    pub max_spend_per_tx: u64,
    /// Maximum cumulative Sompi permitted within a rolling 24-hour window.
    pub daily_velocity_limit: u64,
    /// Whitelist of authorized recipient addresses or contract hashes.
    /// If empty, transactions may target any destination up to spend limits.
    pub contract_whitelist: Vec<String>,
    /// UNIX timestamp (seconds) when this session key expires.
    pub expires_at: u64,
    /// Cumulative Sompi spent in the active 24-hour window.
    pub current_daily_spent: u64,
    /// UNIX timestamp (seconds) when the active 24-hour window started.
    pub daily_window_start: u64,
}

impl SessionPolicy {
    /// Creates a new SessionPolicy with explicit limits and lifetime in seconds.
    pub fn new(
        max_spend_per_tx: u64,
        daily_velocity_limit: u64,
        contract_whitelist: Vec<String>,
        ttl_seconds: u64,
        now: u64,
    ) -> Self {
        Self {
            max_spend_per_tx,
            daily_velocity_limit,
            contract_whitelist,
            expires_at: now.saturating_add(ttl_seconds),
            current_daily_spent: 0,
            daily_window_start: now,
        }
    }

    /// Read-only pre-flight check to see if an intended spend is allowed by policy.
    pub fn check_spend(&self, amount: u64, recipient: &str, now: u64) -> Result<(), SessionPolicyError> {
        // 1. Check expiration
        if now >= self.expires_at {
            return Err(SessionPolicyError::Expired {
                expires_at: self.expires_at,
                current_time: now,
            });
        }

        // 2. Check per-transaction limit
        if amount > self.max_spend_per_tx {
            return Err(SessionPolicyError::ExceedsPerTxLimit {
                limit: self.max_spend_per_tx,
                requested: amount,
            });
        }

        // 3. Calculate effective daily spent (simulate 24-hour window rollover)
        let effective_daily = if now.saturating_sub(self.daily_window_start) >= 86400 {
            0
        } else {
            self.current_daily_spent
        };

        // 4. Check cumulative daily velocity limit
        let new_daily = effective_daily.checked_add(amount).ok_or(SessionPolicyError::Overflow)?;
        if new_daily > self.daily_velocity_limit {
            return Err(SessionPolicyError::ExceedsDailyLimit {
                limit: self.daily_velocity_limit,
                attempted_total: new_daily,
            });
        }

        // 5. Check destination whitelist
        if !self.contract_whitelist.is_empty()
            && !self.contract_whitelist.iter().any(|allowed| allowed == recipient)
        {
            return Err(SessionPolicyError::DestinationNotWhitelisted {
                recipient: recipient.to_string(),
            });
        }

        Ok(())
    }

    /// Validates an outbound transaction and records the spent amount into the daily velocity meter.
    pub fn validate_and_record_spend(
        &mut self,
        amount: u64,
        recipient: &str,
        now: u64,
    ) -> Result<(), SessionPolicyError> {
        // Perform standard checks
        self.check_spend(amount, recipient, now)?;

        // Rollover 24h window if elapsed
        if now.saturating_sub(self.daily_window_start) >= 86400 {
            self.current_daily_spent = 0;
            self.daily_window_start = now;
        }

        // Increment daily spend
        let new_daily = self.current_daily_spent.checked_add(amount).ok_or(SessionPolicyError::Overflow)?;
        self.current_daily_spent = new_daily;

        Ok(())
    }
}

/// An ephemeral agent session certificate issued by a master wallet.
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct SessionCertificate {
    /// Unique identifier for this session.
    pub session_id: String,
    /// Human-readable agent or service label (e.g. "trading-sentinel-v1").
    pub agent_label: String,
    /// Hex-encoded public key of the ephemeral child session key.
    pub public_key: String,
    /// The enforced spending policy.
    pub policy: SessionPolicy,
    /// Master public key that authorized this session certificate.
    pub master_public_key: String,
    /// Optional Schnorr signature from the master key over `hash(session_id || public_key || policy)`.
    pub master_signature: Option<String>,
}

impl SessionCertificate {
    /// Creates a new SessionCertificate.
    pub fn new(
        session_id: String,
        agent_label: String,
        public_key: String,
        master_public_key: String,
        policy: SessionPolicy,
    ) -> Self {
        Self {
            session_id,
            agent_label,
            public_key,
            policy,
            master_public_key,
            master_signature: None,
        }
    }

    /// Canonical payload string used for signing and verification.
    pub fn payload_for_signing(&self) -> String {
        format!(
            "session_id:{}\nagent_label:{}\npublic_key:{}\nmax_per_tx:{}\ndaily_cap:{}\nwhitelist:{}\nexpires_at:{}",
            self.session_id,
            self.agent_label,
            self.public_key,
            self.policy.max_spend_per_tx,
            self.policy.daily_velocity_limit,
            self.policy.contract_whitelist.join(","),
            self.policy.expires_at,
        )
    }

    /// Signs this certificate with the master private key using Schnorr personal message signing.
    pub fn sign(&mut self, master_privkey: &[u8; 32]) -> Result<(), SessionPolicyError> {
        let payload = self.payload_for_signing();
        let sig = crate::message::sign_message(
            &crate::message::PersonalMessage(&payload),
            master_privkey,
            &crate::message::SignMessageOptions { no_aux_rand: true },
        )
        .map_err(|_| SessionPolicyError::InvalidSignature)?;

        self.master_signature = Some(faster_hex::hex_string(&sig));
        Ok(())
    }

    /// Verifies the master signature against the master public key and policy payload.
    pub fn verify(&self) -> Result<(), SessionPolicyError> {
        let sig_hex = self.master_signature.as_ref().ok_or(SessionPolicyError::InvalidSignature)?;
        let mut sig_bytes = [0u8; 64];
        faster_hex::hex_decode(sig_hex.as_bytes(), &mut sig_bytes).map_err(|_| SessionPolicyError::InvalidSignature)?;

        let mut master_pk_bytes = [0u8; 32];
        faster_hex::hex_decode(self.master_public_key.as_bytes(), &mut master_pk_bytes)
            .map_err(|_| SessionPolicyError::InvalidSignature)?;
        let xonly_pk =
            secp256k1::XOnlyPublicKey::from_slice(&master_pk_bytes).map_err(|_| SessionPolicyError::InvalidSignature)?;

        let payload = self.payload_for_signing();
        crate::message::verify_message(
            &crate::message::PersonalMessage(&payload),
            &sig_bytes.to_vec(),
            &xonly_pk,
        )
        .map_err(|_| SessionPolicyError::InvalidSignature)?;

        Ok(())
    }
}

/// In-memory scoped session key container holding the ephemeral private key and policy.
pub struct ScopedSessionKey {
    pub certificate: SessionCertificate,
    private_key: Secret,
}

impl ScopedSessionKey {
    /// Creates a new ScopedSessionKey wrapping the ephemeral secret key.
    pub fn new(certificate: SessionCertificate, private_key_bytes: Vec<u8>) -> Self {
        Self {
            certificate,
            private_key: Secret::new(private_key_bytes),
        }
    }

    /// Generates a new ephemeral Secp256k1 keypair and builds a ScopedSessionKey with an unsigned certificate.
    pub fn generate(
        session_id: String,
        agent_label: String,
        master_public_key: String,
        policy: SessionPolicy,
    ) -> (Self, String) {
        let secp = secp256k1::Secp256k1::new();
        let (secret_key, public_key) = secp.generate_keypair(&mut rand::thread_rng());
        let (xonly, _) = public_key.x_only_public_key();
        let pubkey_hex = faster_hex::hex_string(&xonly.serialize());
        let privkey_bytes = secret_key.secret_bytes().to_vec();
        let privkey_hex = faster_hex::hex_string(&privkey_bytes);

        let cert = SessionCertificate::new(
            session_id,
            agent_label,
            pubkey_hex,
            master_public_key,
            policy,
        );

        let scoped_key = Self::new(cert, privkey_bytes);
        (scoped_key, privkey_hex)
    }

    /// Access the underlying private key bytes for authorized signing operations.
    pub fn private_key_bytes(&self) -> &[u8] {
        self.private_key.as_ref()
    }

    /// Pre-flight validation against the session's policy before signing.
    pub fn authorize_tx(&mut self, amount: u64, recipient: &str, now: u64) -> Result<(), SessionPolicyError> {
        self.certificate.policy.validate_and_record_spend(amount, recipient, now)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_per_tx_spend_limit_enforced() {
        let now = 1726000000;
        let mut policy = SessionPolicy::new(
            50_000_000,   // max 0.5 ZYAN per tx
            500_000_000,  // max 5.0 ZYAN per day
            vec![],       // open whitelist
            86400,        // 24h TTL
            now,
        );

        // Valid spend: 40,000,000 <= 50,000,000
        assert!(policy.validate_and_record_spend(40_000_000, "zyanya:qqtest", now).is_ok());

        // Violating spend: 60,000,000 > 50,000,000
        let res = policy.validate_and_record_spend(60_000_000, "zyanya:qqtest", now);
        assert_eq!(
            res,
            Err(SessionPolicyError::ExceedsPerTxLimit {
                limit: 50_000_000,
                requested: 60_000_000,
            })
        );
    }

    #[test]
    fn test_daily_velocity_limit_and_rollover() {
        let now = 1726000000;
        let mut policy = SessionPolicy::new(
            50_000_000,   // max 0.5 ZYAN per tx
            100_000_000,  // max 1.0 ZYAN per day
            vec![],
            86400 * 7,    // 7-day TTL
            now,
        );

        // Spend 1: 50M
        assert!(policy.validate_and_record_spend(50_000_000, "zyanya:qq1", now).is_ok());
        assert_eq!(policy.current_daily_spent, 50_000_000);

        // Spend 2: 50M (cumulative 100M == daily limit)
        assert!(policy.validate_and_record_spend(50_000_000, "zyanya:qq2", now + 100).is_ok());
        assert_eq!(policy.current_daily_spent, 100_000_000);

        // Spend 3: Attempt another 10M within same day -> Rejected
        let res = policy.validate_and_record_spend(10_000_000, "zyanya:qq3", now + 200);
        assert_eq!(
            res,
            Err(SessionPolicyError::ExceedsDailyLimit {
                limit: 100_000_000,
                attempted_total: 110_000_000,
            })
        );

        // Advance time by 86,401 seconds (next day rollover)
        let next_day = now + 86401;
        assert!(policy.validate_and_record_spend(40_000_000, "zyanya:qq4", next_day).is_ok());
        assert_eq!(policy.current_daily_spent, 40_000_000);
    }

    #[test]
    fn test_expiration_enforced() {
        let now = 1726000000;
        let mut policy = SessionPolicy::new(50_000_000, 500_000_000, vec![], 3600, now);

        // Before expiry: OK
        assert!(policy.validate_and_record_spend(10_000_000, "zyanya:qq1", now + 1800).is_ok());

        // After expiry: Rejected
        let res = policy.validate_and_record_spend(10_000_000, "zyanya:qq1", now + 3601);
        assert_eq!(
            res,
            Err(SessionPolicyError::Expired {
                expires_at: now + 3600,
                current_time: now + 3601,
            })
        );
    }

    #[test]
    fn test_contract_whitelist_enforced() {
        let now = 1726000000;
        let authorized_dex = "zyanya:qq8fdex000000000000000000000000000000000".to_string();
        let authorized_bonding = "zyanya:qq2abonding00000000000000000000000000000".to_string();

        let mut policy = SessionPolicy::new(
            50_000_000,
            500_000_000,
            vec![authorized_dex.clone(), authorized_bonding.clone()],
            86400,
            now,
        );

        // Allowed target
        assert!(policy.validate_and_record_spend(10_000_000, &authorized_dex, now).is_ok());

        // Unauthorized target -> Rejected
        let unauthorized_target = "zyanya:qqattacker00000000000000000000000000000";
        let res = policy.validate_and_record_spend(10_000_000, unauthorized_target, now);
        assert_eq!(
            res,
            Err(SessionPolicyError::DestinationNotWhitelisted {
                recipient: unauthorized_target.to_string(),
            })
        );
    }

    #[test]
    fn test_certificate_serialization() {
        let now = 1726000000;
        let policy = SessionPolicy::new(10_000_000, 100_000_000, vec![], 3600, now);
        let cert = SessionCertificate::new(
            "sess-alpha-001".to_string(),
            "trading-bot".to_string(),
            "02abcd1234...".to_string(),
            "03ef5678...".to_string(),
            policy,
        );

        let json = serde_json::to_string(&cert).expect("Failed to serialize");
        let deserialized: SessionCertificate = serde_json::from_str(&json).expect("Failed to deserialize");
        assert_eq!(cert, deserialized);
    }

    #[test]
    fn test_certificate_signing_and_verification() {
        let now = 1726000000;
        let secp = secp256k1::Secp256k1::new();
        let (master_sk, master_pk) = secp.generate_keypair(&mut rand::thread_rng());
        let (master_xonly, _) = master_pk.x_only_public_key();
        let master_pk_hex = faster_hex::hex_string(&master_xonly.serialize());
        let master_sk_bytes = master_sk.secret_bytes();

        let policy = SessionPolicy::new(25_000_000, 250_000_000, vec!["zyanya:dex".to_string()], 86400, now);
        let (mut scoped_key, _ephemeral_privkey_hex) = ScopedSessionKey::generate(
            "sess-crypt-001".to_string(),
            "defi-agent".to_string(),
            master_pk_hex,
            policy,
        );

        // Initially unsigned: verify fails
        assert_eq!(scoped_key.certificate.verify(), Err(SessionPolicyError::InvalidSignature));

        // Sign with master key
        assert!(scoped_key.certificate.sign(&master_sk_bytes).is_ok());
        assert!(scoped_key.certificate.master_signature.is_some());

        // Verify valid certificate passes
        assert!(scoped_key.certificate.verify().is_ok());
    }

    #[test]
    fn test_certificate_tamper_detection() {
        let now = 1726000000;
        let secp = secp256k1::Secp256k1::new();
        let (master_sk, master_pk) = secp.generate_keypair(&mut rand::thread_rng());
        let (master_xonly, _) = master_pk.x_only_public_key();
        let master_pk_hex = faster_hex::hex_string(&master_xonly.serialize());
        let master_sk_bytes = master_sk.secret_bytes();

        let policy = SessionPolicy::new(10_000_000, 100_000_000, vec!["zyanya:dex".to_string()], 3600, now);
        let (mut scoped_key, _) = ScopedSessionKey::generate(
            "sess-tamper-001".to_string(),
            "trading-bot".to_string(),
            master_pk_hex,
            policy,
        );

        assert!(scoped_key.certificate.sign(&master_sk_bytes).is_ok());
        assert!(scoped_key.certificate.verify().is_ok());

        // Tamper with spend limit
        scoped_key.certificate.policy.max_spend_per_tx = 999_999_999;
        assert_eq!(scoped_key.certificate.verify(), Err(SessionPolicyError::InvalidSignature));

        // Restore spend limit, tamper with expiry
        scoped_key.certificate.policy.max_spend_per_tx = 10_000_000;
        scoped_key.certificate.policy.expires_at = now + 999_999;
        assert_eq!(scoped_key.certificate.verify(), Err(SessionPolicyError::InvalidSignature));
    }
}
