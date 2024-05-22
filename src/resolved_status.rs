use std::fmt::Display;
use std::net::IpAddr;

use crate::dns::QueryType;
use crate::dns::ResultCode;

/// Represents the result of a name resolution
pub enum ResolvedStatus {
    /// Indicates that the FQDN is not listed in the allowlist and has been denied
    Deny(QueryType, String, ResultCode),
    /// Indicates that the FQDN is listed in the allowlist and has been resolved
    Allow(QueryType, String, Vec<ResolvedData>),
    /// Indicates that the FQDN is listed in the allowlist but the name resolution failed
    AllowButError(QueryType, String, ResultCode),
}

impl Display for ResolvedStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Deny(qtype, name, code) => write!(f, "[Deny] <{qtype}> {name}: {code}"),
            Self::Allow(qtype, name, data) => write!(
                f,
                "[Allow] <{qtype}> {name} {}",
                data.iter()
                    .map(|x| x.to_string())
                    .collect::<Vec<_>>()
                    .join(", ")
            ),
            Self::AllowButError(qtype, name, code) => write!(f, "[Allow] <{qtype}> {name}: {code}"),
        }
    }
}

pub enum ResolvedData {
    IpAddr(IpAddr),
    String(String),
}

impl Display for ResolvedData {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::IpAddr(v) => write!(f, "{v}"),
            Self::String(v) => write!(f, "{v}"),
        }
    }
}
