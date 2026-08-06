/// RPC authentication configuration with optional credentials.
///
/// When credentials are set (both user and password are non-empty), the auth
/// interceptor checks incoming requests for valid Basic auth.
/// When credentials are empty (default), all requests are allowed (backward compatible).
#[derive(Clone, Debug, Default)]
pub struct RpcAuthConfig {
    pub user: String,
    pub password: String,
}

impl RpcAuthConfig {
    pub fn new(user: String, password: String) -> Self {
        Self { user, password }
    }

    pub fn is_auth_required(&self) -> bool {
        !self.user.is_empty() && !self.password.is_empty()
    }

    pub fn validate_credentials(&self, user: &str, pass: &str) -> bool {
        self.user == user && self.password == pass
    }

    /// Validate a Basic auth header value (e.g. "Basic YWRtaW46c2VjcmV0")
    /// Returns Ok(()) if valid or auth not required, Err(msg) if invalid
    pub fn validate_basic_auth(&self, auth_header: Option<&str>) -> Result<(), String> {
        if !self.is_auth_required() {
            return Ok(());
        }

        match auth_header {
            Some(header) if header.starts_with("Basic ") => {
                let encoded = &header[6..];
                if let Ok(decoded) = base64_decode(encoded) {
                    let expected = format!("{}:{}", self.user, self.password);
                    if decoded == expected {
                        return Ok(());
                    }
                }
                Err("Invalid credentials".to_string())
            }
            _ => Err("Missing or invalid Authorization header".to_string()),
        }
    }
}

/// Simple base64 decoder for auth headers (avoids adding a base64 dependency)
fn base64_decode(input: &str) -> Result<String, ()> {
    const TABLE: &[u8; 64] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";

    let input = input.trim_end_matches('=');
    let bytes = input.as_bytes();
    let mut result = Vec::with_capacity(input.len() * 3 / 4);

    for chunk in bytes.chunks(4) {
        let mut buf = [0u8; 4];
        for (i, &b) in chunk.iter().enumerate() {
            buf[i] = TABLE.iter().position(|&t| t == b).ok_or(())? as u8;
        }
        result.push((buf[0] << 2) | (buf[1] >> 4));
        if chunk.len() > 2 {
            result.push((buf[1] << 4) | (buf[2] >> 2));
        }
        if chunk.len() > 3 {
            result.push((buf[2] << 6) | buf[3]);
        }
    }

    String::from_utf8(result).map_err(|_| ())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_no_auth_required_when_empty() {
        let config = RpcAuthConfig::default();
        assert!(!config.is_auth_required());
    }

    #[test]
    fn test_auth_required_when_set() {
        let config = RpcAuthConfig::new("admin".to_string(), "secret".to_string());
        assert!(config.is_auth_required());
    }

    #[test]
    fn test_validate_correct_credentials() {
        let config = RpcAuthConfig::new("admin".to_string(), "secret".to_string());
        assert!(config.validate_credentials("admin", "secret"));
    }

    #[test]
    fn test_validate_wrong_credentials() {
        let config = RpcAuthConfig::new("admin".to_string(), "secret".to_string());
        assert!(!config.validate_credentials("admin", "wrong"));
        assert!(!config.validate_credentials("wrong", "secret"));
    }

    #[test]
    fn test_base64_decode_basic() {
        // base64("admin:secret") = "YWRtaW46c2VjcmV0"
        assert_eq!(base64_decode("YWRtaW46c2VjcmV0").unwrap(), "admin:secret");
    }

    #[test]
    fn test_validate_basic_auth_no_auth_required() {
        let config = RpcAuthConfig::default();
        assert!(config.validate_basic_auth(None).is_ok());
    }

    #[test]
    fn test_validate_basic_auth_correct() {
        let config = RpcAuthConfig::new("admin".to_string(), "secret".to_string());
        assert!(config.validate_basic_auth(Some("Basic YWRtaW46c2VjcmV0")).is_ok());
    }

    #[test]
    fn test_validate_basic_auth_missing() {
        let config = RpcAuthConfig::new("admin".to_string(), "secret".to_string());
        assert!(config.validate_basic_auth(None).is_err());
    }

    #[test]
    fn test_validate_basic_auth_wrong() {
        let config = RpcAuthConfig::new("admin".to_string(), "secret".to_string());
        assert!(config.validate_basic_auth(Some("Basic d3Jvbmc6Y3JlZHM=")).is_err());
    }
}