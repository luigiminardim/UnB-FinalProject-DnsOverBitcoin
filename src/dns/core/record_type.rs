#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u16)]
pub enum RecordType {
    A,

    /// Name server record
    NS,

    /// Canonical name record
    Cname,

    /// Mail exchange record
    Mx,

    /// IPv6 address record (RFC 3596)
    Aaaa,

    Unknown(u16),
}

impl From<u16> for RecordType {
    fn from(value: u16) -> Self {
        match value {
            1 => RecordType::A,
            2 => RecordType::NS,
            5 => RecordType::Cname,
            15 => RecordType::Mx,
            28 => RecordType::Aaaa,
            _ => RecordType::Unknown(value),
        }
    }
}

impl From<RecordType> for u16 {
    fn from(value: RecordType) -> Self {
        match value {
            RecordType::A => 1,
            RecordType::NS => 2,
            RecordType::Cname => 5,
            RecordType::Mx => 15,
            RecordType::Aaaa => 28,
            RecordType::Unknown(value) => value,
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
        assert_eq!(RecordType::from(2), RecordType::NS);
        assert_eq!(RecordType::from(5), RecordType::Cname);
        assert_eq!(RecordType::from(15), RecordType::Mx);
        assert_eq!(RecordType::from(28), RecordType::Aaaa);
    }

    #[test]
    fn test_type_into_u16() {
        assert_eq!(u16::from(RecordType::Unknown(0)), 0);
        assert_eq!(u16::from(RecordType::A), 1);
        assert_eq!(u16::from(RecordType::NS), 2);
        assert_eq!(u16::from(RecordType::Cname), 5);
        assert_eq!(u16::from(RecordType::Mx), 15);
        assert_eq!(u16::from(RecordType::Aaaa), 28);
    }
}
