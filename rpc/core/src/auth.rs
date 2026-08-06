use serde::{Deserialize, Serialize};

/// Configuration and validation helper for RPC authentication credentials.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct RpcAuthConfig {
    pub rpc_user: Option<String>,
    pub rpc_pass: Option<String>,
}

impl RpcAuthConfig {
    /// Creates a new `RpcAuthConfig` with the provided username and password.
    pub fn new(rpc_user: Option<String>, rpc_pass: Option<String>) -> Self {
        Self { rpc_user, rpc_pass }
    }

    /// Returns `true` if RPC authentication is required.
    pub fn is_auth_required(&self) -> bool {
        self.rpc_user.is_some() || self.rpc_pass.is_some()
    }

    /// Validates the provided username and password against configured credentials.
    /// Returns `true` if authentication is not required, OR if credentials match.
    pub fn validate(&self, user: &str, pass: &str) -> bool {
        if !self.is_auth_required() {
            return true;
        }

        let user_ok = self.rpc_user.as_deref().map_or(true, |u| u == user);
        let pass_ok = self.rpc_pass.as_deref().map_or(true, |p| p == pass);

        user_ok && pass_ok
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_rpc_auth_config_default_disabled() {
        let auth = RpcAuthConfig::default();
        assert!(!auth.is_auth_required());
        assert!(auth.validate("any_user", "any_pass"));
    }

    #[test]
    fn test_rpc_auth_config_validation() {
        let auth = RpcAuthConfig::new(Some("admin".to_string()), Some("secret123".to_string()));
        assert!(auth.is_auth_required());
        assert!(auth.validate("admin", "secret123"));
        assert!(!auth.validate("admin", "wrongpass"));
        assert!(!auth.validate("wronguser", "secret123"));
        assert!(!auth.validate("wronguser", "wrongpass"));
    }

    #[test]
    fn test_rpc_auth_config_user_only() {
        let auth = RpcAuthConfig::new(Some("admin".to_string()), None);
        assert!(auth.is_auth_required());
        assert!(auth.validate("admin", "anything"));
        assert!(!auth.validate("guest", "anything"));
    }

    #[test]
    fn test_rpc_auth_config_pass_only() {
        let auth = RpcAuthConfig::new(None, Some("secret123".to_string()));
        assert!(auth.is_auth_required());
        assert!(auth.validate("anything", "secret123"));
        assert!(!auth.validate("anything", "wrongpass"));
    }
}
