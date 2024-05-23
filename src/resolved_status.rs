use crate::dns::ResultCode;
use crate::resolved_data::ResolvedData;
use std::fmt::Display;

/// Represents the result of a name resolution
pub enum ResolvedStatus {
    /// Indicates that the FQDN is not listed in the allowlist and has been denied
    Deny(ResolvedData, ResultCode),
    /// Indicates that the FQDN is listed in the allowlist and has been resolved
    Allow(ResolvedData),
    /// Indicates that the FQDN is listed in the allowlist but the name resolution failed
    AllowButError(ResolvedData, ResultCode),
    /// Indicates that the name resolution was performed without checking the allowlist
    NoCheck(ResolvedData),
    /// Indicates that the name resolution failed without checking the allowlist
    NoCheckButError(ResolvedData, ResultCode),
}

impl ResolvedStatus {
    pub fn pretty_fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Deny(v, code) => write!(f, "[Deny] <{}> {}: {code}", v.req_qtype, v.req_name),
            Self::AllowButError(v, code) => {
                write!(f, "[Allow] <{}> {}: {code}", v.req_qtype, v.req_name)
            }
            Self::Allow(v) => {
                write!(f, "[Allow] ")?;
                v.pretty_fmt(f)?;
                Ok(())
            }
            Self::NoCheck(v) => {
                write!(f, "[NoCheck] ")?;
                v.pretty_fmt(f)?;
                Ok(())
            }
            Self::NoCheckButError(v, code) => {
                write!(f, "[NoCheck] <{}> {}: {code}", v.req_qtype, v.req_name)
            }
        }
    }

    pub(super) fn into_nocheck(self) -> ResolvedStatus {
        match self {
            Self::Allow(v) => ResolvedStatus::NoCheck(v),
            Self::AllowButError(v, code) => ResolvedStatus::NoCheckButError(v, code),
            v => v,
        }
    }
}

impl Display for ResolvedStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.pretty_fmt(f)
    }
}
