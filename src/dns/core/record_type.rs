#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u16)]
pub enum RecordType {
    A,

    /// IPv6 address record (RFC 3596)
    AAAA,

    Unknown(u16),
}

impl From<u16> for RecordType {
    fn from(value: u16) -> Self {
        match value {
            1 => RecordType::A,
            28 => RecordType::AAAA,
            _ => RecordType::Unknown(value),
        }
    }
}

impl From<RecordType> for u16 {
    fn from(value: RecordType) -> Self {
        match value {
            RecordType::A => 1,
            RecordType::AAAA => 28,
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
        assert_eq!(RecordType::from(28), RecordType::AAAA);
        assert_eq!(RecordType::from(2), RecordType::Unknown(2));
    }

    #[test]
    fn test_type_into_u16() {
        assert_eq!(u16::from(RecordType::Unknown(0)), 0);
        assert_eq!(u16::from(RecordType::A), 1);
        assert_eq!(u16::from(RecordType::AAAA), 28);
        assert_eq!(u16::from(RecordType::Unknown(2)), 2);
    }
}
