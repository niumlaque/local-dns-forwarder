#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ResultCode {
    /// No Error
    NoError = 0,

    /// Format Error
    FormErr = 1,

    /// Server Failure
    ServFail = 2,

    /// Non-Existent Domain
    NXDomain = 3,

    /// Not Implemented
    NotImp = 4,

    /// Query Refused
    Refused = 5,

    /// Name Exists when it should not
    YXDomain = 6,

    /// RR Set Exists when it should not
    YXRRSet = 7,

    /// RR Set that should exist does not
    NXRRSet = 8,

    /// Server Not Authoritative for zone
    NotAuth = 9,

    /// Name not contained in zone
    NotZone = 10,

    /// DSO-TYPE Not Implemented
    DSOTYPENI = 11,

    /// Bad OPT Version
    BADVERS = 16,

    /// Key not recognized
    BADKEY = 17,

    /// Signature out of time window
    BADTIME = 18,

    /// Bad TKEY Mode
    BADMODE = 19,

    /// Duplicate key name
    BADNAME = 20,

    /// Algorithm not supported
    BADALG = 21,

    /// Bad Truncation
    BADTRUNC = 22,

    /// Bad/missing Server Cookie
    BADCOOKIE = 23,
}

impl From<u8> for ResultCode {
    fn from(value: u8) -> Self {
        use ResultCode::*;
        match value {
            0 => NoError,
            1 => FormErr,
            2 => ServFail,
            3 => NXDomain,
            4 => NotImp,
            5 => Refused,
            6 => YXDomain,
            7 => YXRRSet,
            8 => NXRRSet,
            9 => NotAuth,
            10 => NotZone,
            11 => DSOTYPENI,
            16 => BADVERS,
            17 => BADKEY,
            18 => BADTIME,
            19 => BADMODE,
            20 => BADNAME,
            21 => BADALG,
            22 => BADTRUNC,
            23 => BADCOOKIE,
            _ => NoError,
        }
    }
}

impl std::fmt::Display for ResultCode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        use ResultCode::*;
        match self {
            NoError => write!(f, "No Error"),
            FormErr => write!(f, "Form Error"),
            ServFail => write!(f, "Server Failure"),
            NXDomain => write!(f, "Non-Existent Domain"),
            NotImp => write!(f, "Not Implemented"),
            Refused => write!(f, "Query Refused"),
            YXDomain => write!(f, "Name Exists when it should not"),
            YXRRSet => write!(f, "RR Set Exists when it should not"),
            NXRRSet => write!(f, "RR Set that should exist does not"),
            NotAuth => write!(f, "Server Not Authoritative for zone"),
            NotZone => write!(f, "Name not contained in zone"),
            DSOTYPENI => write!(f, "DSO-TYPE Not Implemented"),
            BADVERS => write!(f, "Bad OPT Version"),
            BADKEY => write!(f, "Key not recognized"),
            BADTIME => write!(f, "Signature out of time window"),
            BADMODE => write!(f, "Bad TKEY Mode"),
            BADNAME => write!(f, "Duplicate key name"),
            BADALG => write!(f, "Algorithm not supported"),
            BADTRUNC => write!(f, "Bad Truncation"),
            BADCOOKIE => write!(f, "Bad/missing Server Cookie"),
        }
    }
}
