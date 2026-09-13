use std::time::{SystemTime, UNIX_EPOCH};
use zyanya_addresses::{Address, Version};
use zyanya_wallet_core::{
    account::{BIP32_ACCOUNT_KIND, KEYPAIR_ACCOUNT_KIND},
    session::{ScopedSessionKey, SessionCertificate, SessionPolicy},
};

use crate::imports::*;

#[derive(Default, Handler)]
#[help("Manage scoped agent session keys and spend policies (Track 9)")]
pub struct Session;

impl Session {
    async fn main(self: Arc<Self>, ctx: &Arc<dyn Context>, mut argv: Vec<String>, _cmd: &str) -> Result<()> {
        let ctx = ctx.clone().downcast_arc::<ZyanyaCli>()?;

        if argv.is_empty() {
            return self.display_help(ctx, argv).await;
        }

        let action = argv.remove(0);
        match action.as_str() {
            "create" => {
                self.create(ctx, argv).await?;
            }
            "verify" => {
                self.verify(ctx, argv).await?;
            }
            "check" => {
                self.check(ctx, argv).await?;
            }
            v => {
                tprintln!(ctx, "Unknown session action: '{v}'\n");
                return self.display_help(ctx, argv).await;
            }
        }

        Ok(())
    }

    async fn display_help(self: Arc<Self>, ctx: Arc<ZyanyaCli>, _argv: Vec<String>) -> Result<()> {
        ctx.term().help(
            &[
                (
                    "session create [flags]",
                    "Generate and sign a scoped agent session key.\n\
                     Flags:\n  \
                       --max-per-tx <sompi>  (default: 50,000,000 = 0.5 ZYAN)\n  \
                       --daily-cap <sompi>   (default: 500,000,000 = 5.0 ZYAN)\n  \
                       --whitelist <addrs>   (comma-separated contract whitelist)\n  \
                       --expiry <seconds>    (TTL in seconds, default: 86400 = 24h)\n  \
                       --agent <label>       (agent identifier, default: autonomous-agent)\n  \
                       --out <file>          (output certificate JSON path)",
                ),
                (
                    "session verify <cert_json_or_file>",
                    "Cryptographically verify a session certificate against its master signature.",
                ),
                (
                    "session check <cert_file> <sompi> <recipient>",
                    "Perform pre-flight policy validation against spending limits and whitelist.",
                ),
            ],
            None,
        )?;

        Ok(())
    }

    async fn create(self: Arc<Self>, ctx: Arc<ZyanyaCli>, argv: Vec<String>) -> Result<()> {
        if !ctx.wallet().is_open() {
            return Err(Error::WalletIsNotOpen);
        }

        let mut max_per_tx: u64 = 50_000_000;       // 0.5 ZYAN
        let mut daily_cap: u64 = 500_000_000;       // 5.0 ZYAN
        let mut whitelist_vec: Vec<String> = Vec::new();
        let mut ttl_seconds: u64 = 86_400;          // 24 hours
        let mut agent_label = "autonomous-agent".to_string();
        let mut out_file: Option<String> = None;

        let mut iter = argv.into_iter();
        while let Some(arg) = iter.next() {
            match arg.as_str() {
                "--max-per-tx" => {
                    if let Some(val) = iter.next() {
                        max_per_tx = val.parse::<u64>().map_err(|_| Error::custom("Invalid --max-per-tx value"))?;
                    }
                }
                "--daily-cap" => {
                    if let Some(val) = iter.next() {
                        daily_cap = val.parse::<u64>().map_err(|_| Error::custom("Invalid --daily-cap value"))?;
                    }
                }
                "--whitelist" => {
                    if let Some(val) = iter.next() {
                        whitelist_vec = val.split(',').map(|s| s.trim().to_string()).filter(|s| !s.is_empty()).collect();
                    }
                }
                "--expiry" | "--ttl" => {
                    if let Some(val) = iter.next() {
                        ttl_seconds = val.parse::<u64>().map_err(|_| Error::custom("Invalid --expiry value"))?;
                    }
                }
                "--agent" => {
                    if let Some(val) = iter.next() {
                        agent_label = val;
                    }
                }
                "--out" => {
                    if let Some(val) = iter.next() {
                        out_file = Some(val);
                    }
                }
                other => {
                    tprintln!(ctx, "Warning: unknown flag '{other}' ignored");
                }
            }
        }

        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map_err(|_| Error::custom("System clock before UNIX epoch"))?
            .as_secs();

        // 1. Get current account address and its public key
        let account = ctx.account().await?;
        let receive_addr = account.receive_address()?;
        if receive_addr.version != Version::PubKey {
            return Err(Error::custom("Unsupported address type: master account must use PubKey version"));
        }

        let master_pubkey_hex = faster_hex::hex_string(&receive_addr.payload[0..32]);

        // 2. Fetch master private key for signing
        let master_privkey = self.get_address_private_key(&ctx, receive_addr).await?;

        // 3. Generate session ID and policy
        let session_id = format!("sess-{:x}", now);
        let policy = SessionPolicy::new(max_per_tx, daily_cap, whitelist_vec.clone(), ttl_seconds, now);

        // 4. Generate ephemeral keypair & wrap into ScopedSessionKey
        let (mut scoped_key, privkey_hex) = ScopedSessionKey::generate(
            session_id.clone(),
            agent_label.clone(),
            master_pubkey_hex.clone(),
            policy.clone(),
        );

        // 5. Sign the certificate with master key
        scoped_key
            .certificate
            .sign(&master_privkey)
            .map_err(|e| Error::custom(format!("Failed to sign session certificate: {e}")))?;

        // Verify immediately
        scoped_key
            .certificate
            .verify()
            .map_err(|e| Error::custom(format!("Self-verification of session certificate failed: {e}")))?;

        let cert_json = serde_json::to_string_pretty(&scoped_key.certificate)
            .map_err(|e| Error::custom(format!("Serialization error: {e}")))?;

        tprintln!(ctx, "\n=======================================================");
        tprintln!(ctx, "       🛡️ SCOPED AGENT SESSION KEY GENERATED");
        tprintln!(ctx, "=======================================================");
        tprintln!(ctx, "Session ID:        {}", session_id);
        tprintln!(ctx, "Agent Label:       {}", agent_label);
        tprintln!(ctx, "Master Pubkey:     {}", master_pubkey_hex);
        tprintln!(ctx, "Ephemeral Pubkey:  {}", scoped_key.certificate.public_key);
        tprintln!(ctx, "Per-Tx Spend Max:  {} Sompi ({:.4} ZYAN)", max_per_tx, max_per_tx as f64 / 1e8);
        tprintln!(ctx, "Daily Velocity:    {} Sompi ({:.4} ZYAN)", daily_cap, daily_cap as f64 / 1e8);
        tprintln!(ctx, "Whitelist Entries: {}", if whitelist_vec.is_empty() { "OPEN (Any target)".to_string() } else { whitelist_vec.join(", ") });
        tprintln!(ctx, "Expires At:        {} (TTL: {}s)", scoped_key.certificate.policy.expires_at, ttl_seconds);
        tprintln!(ctx, "Master Signature:  {}", scoped_key.certificate.master_signature.as_deref().unwrap_or("none"));
        tprintln!(ctx, "-------------------------------------------------------");
        tprintln!(ctx, "Agent Ephemeral Private Key (Hex):");
        tprintln!(ctx, "{}", privkey_hex);
        tprintln!(ctx, "=======================================================\n");

        if let Some(ref path) = out_file {
            std::fs::write(path, &cert_json).map_err(|e| Error::custom(format!("Failed to write to {path}: {e}")))?;
            tprintln!(ctx, "Certificate written to: {path}\n");
        }

        Ok(())
    }

    async fn verify(self: Arc<Self>, ctx: Arc<ZyanyaCli>, argv: Vec<String>) -> Result<()> {
        if argv.is_empty() {
            tprintln!(ctx, "Usage: session verify <cert_json_or_file>");
            return Ok(());
        }

        let input = argv.join(" ");
        let cert: SessionCertificate = if std::path::Path::new(&input).exists() {
            let content = std::fs::read_to_string(&input).map_err(|e| Error::custom(format!("Read error: {e}")))?;
            serde_json::from_str(&content).map_err(|e| Error::custom(format!("JSON parse error: {e}")))?
        } else {
            serde_json::from_str(&input).map_err(|e| Error::custom(format!("JSON parse error: {e}")))?
        };

        match cert.verify() {
            Ok(()) => {
                tprintln!(ctx, "\n[✓] Session certificate is VALID.");
                tprintln!(ctx, "Session ID:        {}", cert.session_id);
                tprintln!(ctx, "Agent Label:       {}", cert.agent_label);
                tprintln!(ctx, "Master Pubkey:     {}", cert.master_public_key);
                tprintln!(ctx, "Ephemeral Pubkey:  {}", cert.public_key);
                tprintln!(ctx, "Expires At:        {}", cert.policy.expires_at);
                tprintln!(ctx, "Master Signature:  {}\n", cert.master_signature.as_deref().unwrap_or("none"));
            }
            Err(e) => {
                tprintln!(ctx, "\n[✗] Session certificate verification FAILED: {}\n", e);
                return Err(Error::custom(format!("Verification failed: {e}")));
            }
        }

        Ok(())
    }

    async fn check(self: Arc<Self>, ctx: Arc<ZyanyaCli>, argv: Vec<String>) -> Result<()> {
        if argv.len() < 3 {
            tprintln!(ctx, "Usage: session check <cert_file_or_json> <amount_sompi> <destination>");
            return Ok(());
        }

        let cert_source = &argv[0];
        let amount: u64 = argv[1].parse().map_err(|_| Error::custom("Invalid sompi amount"))?;
        let destination = &argv[2];

        let cert: SessionCertificate = if std::path::Path::new(cert_source).exists() {
            let content = std::fs::read_to_string(cert_source).map_err(|e| Error::custom(format!("Read error: {e}")))?;
            serde_json::from_str(&content).map_err(|e| Error::custom(format!("JSON parse error: {e}")))?
        } else {
            serde_json::from_str(cert_source).map_err(|e| Error::custom(format!("JSON parse error: {e}")))?
        };

        // First verify signature
        cert.verify().map_err(|e| Error::custom(format!("Certificate signature invalid: {e}")))?;

        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map_err(|_| Error::custom("System clock error"))?
            .as_secs();

        match cert.policy.check_spend(amount, destination, now) {
            Ok(()) => {
                tprintln!(ctx, "\n[✓] Spend AUTHORIZED by session policy.");
                tprintln!(ctx, "Requested Amount:  {} Sompi", amount);
                tprintln!(ctx, "Target Recipient:  {}\n", destination);
            }
            Err(e) => {
                tprintln!(ctx, "\n[✗] Spend REJECTED by session policy: {}\n", e);
                return Err(Error::custom(format!("Policy check rejected: {e}")));
            }
        }

        Ok(())
    }

    async fn get_address_private_key(self: &Arc<Self>, ctx: &Arc<ZyanyaCli>, zyanya_address: Address) -> Result<[u8; 32]> {
        let account = ctx.wallet().account()?;

        match account.account_kind().as_ref() {
            BIP32_ACCOUNT_KIND => {
                let (wallet_secret, payment_secret) = ctx.ask_wallet_secret(Some(&account)).await?;
                let keydata = account.prv_key_data(wallet_secret).await?;
                let account = account.clone().as_derivation_capable().expect("Account should support derivation.");

                let (receive, change) = account.derivation().addresses_indexes(&[&zyanya_address])?;
                let private_keys = account.create_private_keys(&keydata, &payment_secret, &receive, &change)?;
                for (address, private_key) in private_keys {
                    if zyanya_address == *address {
                        return Ok(private_key.secret_bytes());
                    }
                }

                Err(Error::custom("Address not found in any derivation path of the account."))
            }
            KEYPAIR_ACCOUNT_KIND => {
                let (wallet_secret, payment_secret) = ctx.ask_wallet_secret(Some(&account)).await?;
                let keydata = account.prv_key_data(wallet_secret).await?;
                let decrypted_privkey = keydata.payload.decrypt(payment_secret.as_ref()).unwrap();
                let secretkey = decrypted_privkey.as_secret_key()?.unwrap();
                Ok(secretkey.secret_bytes())
            }
            _ => Err(Error::custom("Unsupported account type for session key signing.")),
        }
    }
}
