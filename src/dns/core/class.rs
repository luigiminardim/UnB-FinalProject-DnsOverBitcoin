#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Class {
    IN,
    Unknown(u16),
}

impl From<u16> for Class {
    fn from(value: u16) -> Self {
        match value {
            1 => Class::IN,
            value => Class::Unknown(value),
        }
    }
}

impl From<Class> for u16 {
    fn from(value: Class) -> Self {
        match value {
            Class::IN => 1,
            Class::Unknown(value) => value,
        }
    }
}

#[cfg(test)]
mod test_class {
    use super::*;

    #[test]
    fn test_class_from_u16() {
        assert_eq!(Class::from(0), Class::Unknown(0));
        assert_eq!(Class::from(1), Class::IN);
        assert_eq!(Class::from(2), Class::Unknown(2));
    }

    #[test]
    fn test_class_into_u16() {
        assert_eq!(u16::from(Class::Unknown(0)), 0);
        assert_eq!(u16::from(Class::IN), 1);
        assert_eq!(u16::from(Class::Unknown(2)), 2);
    }
}
