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
