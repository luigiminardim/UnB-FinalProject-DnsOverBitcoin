use std::str::FromStr;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u16)]
pub enum RecordType {
    A,

    /// Name server record
    Ns,

    /// Canonical name record
    Cname,

    /// Mail exchange record
    Mx,

    /// Text record
    Txt,

    /// IPv6 address record (RFC 3596)
    Aaaa,

    Unknown(u16),
}

impl From<u16> for RecordType {
    fn from(value: u16) -> Self {
        match value {
            1 => RecordType::A,
            2 => RecordType::Ns,
            5 => RecordType::Cname,
            15 => RecordType::Mx,
            16 => RecordType::Txt,
            28 => RecordType::Aaaa,
            _ => RecordType::Unknown(value),
        }
    }
}

impl From<RecordType> for u16 {
    fn from(value: RecordType) -> Self {
        match value {
            RecordType::A => 1,
            RecordType::Ns => 2,
            RecordType::Cname => 5,
            RecordType::Mx => 15,
            RecordType::Txt => 16,
            RecordType::Aaaa => 28,
            RecordType::Unknown(value) => value,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RecordTypeFromStrErr {
    Invalid,
}

impl FromStr for RecordType {
    type Err = RecordTypeFromStrErr;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "A" => Ok(RecordType::A),
            "NS" => Ok(RecordType::Ns),
            "CNAME" => Ok(RecordType::Cname),
            "MX" => Ok(RecordType::Mx),
            "TXT" => Ok(RecordType::Txt),
            "AAAA" => Ok(RecordType::Aaaa),
            _ => Err(RecordTypeFromStrErr::Invalid),
        }
    }
}

impl ToString for RecordType {
    fn to_string(&self) -> String {
        match self {
            RecordType::A => "A".to_string(),
            RecordType::Ns => "NS".to_string(),
            RecordType::Cname => "CNAME".to_string(),
            RecordType::Mx => "MX".to_string(),
            RecordType::Txt => "TXT".to_string(),
            RecordType::Aaaa => "AAAA".to_string(),
            RecordType::Unknown(value) => value.to_string(),
        }
    }
}

#[cfg(test)]
mod test_type {
    use super::*;

    #[test]
    fn test_type_from_u16() {
        assert_eq!(RecordType::from(0), RecordType::Unknown(0));
        assert_eq!(RecordType::from(1), RecordType::A);
        assert_eq!(RecordType::from(2), RecordType::Ns);
        assert_eq!(RecordType::from(5), RecordType::Cname);
        assert_eq!(RecordType::from(15), RecordType::Mx);
        assert_eq!(RecordType::from(16), RecordType::Txt);
        assert_eq!(RecordType::from(28), RecordType::Aaaa);
    }

    #[test]
    fn test_type_into_u16() {
        assert_eq!(u16::from(RecordType::Unknown(0)), 0);
        assert_eq!(u16::from(RecordType::A), 1);
        assert_eq!(u16::from(RecordType::Ns), 2);
        assert_eq!(u16::from(RecordType::Cname), 5);
        assert_eq!(u16::from(RecordType::Mx), 15);
        assert_eq!(u16::from(RecordType::Txt), 16);
        assert_eq!(u16::from(RecordType::Aaaa), 28);
    }

    #[test]
    fn test_type_from_str() {
        assert_eq!(RecordType::from_str("A").unwrap(), RecordType::A);
        assert_eq!(RecordType::from_str("NS").unwrap(), RecordType::Ns);
        assert_eq!(RecordType::from_str("CNAME").unwrap(), RecordType::Cname);
        assert_eq!(RecordType::from_str("MX").unwrap(), RecordType::Mx);
        assert_eq!(RecordType::from_str("TXT").unwrap(), RecordType::Txt);
        assert_eq!(RecordType::from_str("AAAA").unwrap(), RecordType::Aaaa);
        assert!(RecordType::from_str("INVALID").is_err());
    }

    #[test]
    fn test_type_to_string() {
        assert_eq!(RecordType::A.to_string(), "A");
        assert_eq!(RecordType::Ns.to_string(), "NS");
        assert_eq!(RecordType::Cname.to_string(), "CNAME");
        assert_eq!(RecordType::Mx.to_string(), "MX");
        assert_eq!(RecordType::Txt.to_string(), "TXT");
        assert_eq!(RecordType::Aaaa.to_string(), "AAAA");
        assert_eq!(RecordType::Unknown(100).to_string(), "100");
    }
}
