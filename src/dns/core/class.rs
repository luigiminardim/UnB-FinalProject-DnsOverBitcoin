use std::str::FromStr;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Class {
    In,
    Unknown(u16),
}

impl From<u16> for Class {
    fn from(value: u16) -> Self {
        match value {
            1 => Class::In,
            value => Class::Unknown(value),
        }
    }
}

impl From<Class> for u16 {
    fn from(value: Class) -> Self {
        match value {
            Class::In => 1,
            Class::Unknown(value) => value,
        }
    }
}

pub enum ClassFromStrErr {
    InvalidClass,
}

impl FromStr for Class {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "IN" => Ok(Class::In),
            _ => Err(()),
        }
    }
}

impl ToString for Class {
    fn to_string(&self) -> String {
        match self {
            Class::In => "IN".to_string(),
            Class::Unknown(value) => value.to_string(),
        }
    }
}

#[cfg(test)]
mod test_class {
    use super::*;

    #[test]
    fn test_class_from_u16() {
        assert_eq!(Class::from(0), Class::Unknown(0));
        assert_eq!(Class::from(1), Class::In);
        assert_eq!(Class::from(2), Class::Unknown(2));
    }

    #[test]
    fn test_class_into_u16() {
        assert_eq!(u16::from(Class::Unknown(0)), 0);
        assert_eq!(u16::from(Class::In), 1);
        assert_eq!(u16::from(Class::Unknown(2)), 2);
    }

    #[test]
    fn test_class_from_str() {
        assert_eq!("IN".parse::<Class>(), Ok(Class::In));
        assert_eq!("INVALID".parse::<Class>(), Err(()));
    }

    #[test]
    fn test_class_to_string() {
        assert_eq!(Class::In.to_string(), "IN");
        assert_eq!(Class::Unknown(0).to_string(), "0");
    }
}
