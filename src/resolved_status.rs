use std::net::IpAddr;

use crate::dns::ResultCode;

pub enum ResolvedStatus {
    Deny(String, ResultCode),
    Allow(String, Vec<IpAddr>),
    AllowButError(String, ResultCode),
}
