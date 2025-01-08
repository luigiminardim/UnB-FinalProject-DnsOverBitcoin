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

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_from_u16() {
        assert_eq!(QueryClass::from(1), QueryClass::Class(Class::IN));
        assert_eq!(QueryClass::from(255), QueryClass::Any);
    }

    #[test]
    fn test_into_u16() {
        assert_eq!(u16::from(QueryClass::Class(Class::IN)), 1);
        assert_eq!(u16::from(QueryClass::Any), 255);
    }

    #[test]
    fn test_matches() {
        assert!(QueryClass::Class(Class::IN).matches(Class::IN));
        assert!(!QueryClass::Class(Class::IN).matches(Class::Unknown(0)));
        assert!(QueryClass::Any.matches(Class::IN));
        assert!(QueryClass::Any.matches(Class::Unknown(0)));
    }
}
