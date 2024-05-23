use std::fmt::Display;

#[repr(u16)]
#[derive(Debug, Clone, Copy, PartialEq, PartialOrd, Ord, Eq, Hash)]
pub enum QueryType {
    UNKNOWN(u16),
    /// A host address
    A = 1,

    /// IP6 Address
    AAAA = 28,

    CNAME = 5,

    /// Service locator
    SRV = 33,
}

impl From<QueryType> for u16 {
    fn from(value: QueryType) -> Self {
        use QueryType::*;
        match value {
            UNKNOWN(v) => v,
            A => 1,
            AAAA => 28,
            CNAME => 5,
            SRV => 33,
        }
    }
}

impl From<u16> for QueryType {
    fn from(value: u16) -> Self {
        match value {
            1 => QueryType::A,
            5 => QueryType::CNAME,
            28 => QueryType::AAAA,
            33 => QueryType::SRV,
            _ => QueryType::UNKNOWN(value),
        }
    }
}

impl Display for QueryType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        use QueryType::*;
        match self {
            A => write!(f, "A"),
            AAAA => write!(f, "AAAA"),
            CNAME => write!(f, "CNAME"),
            SRV => write!(f, "SRV"),
            UNKNOWN(v) => write!(f, "UNKNOWN({v})"),
        }
    }
}
