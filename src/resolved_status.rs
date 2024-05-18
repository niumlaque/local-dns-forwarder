use std::net::IpAddr;

use crate::dns::ResultCode;

/// Represents the result of a name resolution
pub enum ResolvedStatus {
    /// Indicates that the FQDN is not listed in the allowlist and has been denied
    Deny(String, ResultCode),
    /// Indicates that the FQDN is listed in the allowlist and has been resolved
    Allow(String, Vec<IpAddr>),
    /// Indicates that the FQDN is listed in the allowlist but the name resolution failed
    AllowButError(String, ResultCode),
}
