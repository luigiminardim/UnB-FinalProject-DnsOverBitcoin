use std::str::FromStr;

use super::Class;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum QueryClass {
    Class(Class),
    Any,
}

impl From<u16> for QueryClass {
    fn from(value: u16) -> Self {
        match value {
            255 => QueryClass::Any,
            value => QueryClass::Class(Class::from(value)),
        }
    }
}

impl From<QueryClass> for u16 {
    fn from(value: QueryClass) -> Self {
        match value {
            QueryClass::Any => 255,
            QueryClass::Class(value) => u16::from(value),
        }
    }
}

impl QueryClass {
    pub fn matches(&self, class: Class) -> bool {
        match self {
            QueryClass::Any => true,
            QueryClass::Class(value) => *value == class,
        }
    }
}

impl FromStr for QueryClass {
    type Err = <Class as FromStr>::Err;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "*" => Ok(QueryClass::Any),
            _ => Class::from_str(s).map(QueryClass::Class),
        }
    }
}

impl ToString for QueryClass {
    fn to_string(&self) -> String {
        match self {
            QueryClass::Class(value) => value.to_string(),
            QueryClass::Any => "*".to_string(),
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_from_u16() {
        assert_eq!(QueryClass::from(1), QueryClass::Class(Class::In));
        assert_eq!(QueryClass::from(255), QueryClass::Any);
    }

    #[test]
    fn test_into_u16() {
        assert_eq!(u16::from(QueryClass::Class(Class::In)), 1);
        assert_eq!(u16::from(QueryClass::Any), 255);
    }

    #[test]
    fn test_matches() {
        assert!(QueryClass::Class(Class::In).matches(Class::In));
        assert!(!QueryClass::Class(Class::In).matches(Class::Unknown(0)));
        assert!(QueryClass::Any.matches(Class::In));
        assert!(QueryClass::Any.matches(Class::Unknown(0)));
    }

    #[test]
    fn test_from_str() {
        // invalid
        assert!(QueryClass::from_str("INVALID").is_err());

        // valid
        assert_eq!(QueryClass::from_str("IN").unwrap(), QueryClass::Class(Class::In));
        assert_eq!(QueryClass::from_str("*").unwrap(), QueryClass::Any);
    }

    #[test]
    fn test_to_string() {
        assert_eq!(QueryClass::Class(Class::In).to_string(), "IN");
        assert_eq!(QueryClass::Any.to_string(), "*");
    }
}
