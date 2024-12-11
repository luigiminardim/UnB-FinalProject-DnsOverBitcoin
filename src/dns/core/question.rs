use super::{Name, QueryClass, QueryType};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Question {
    name: Name,
    record_type: QueryType,
    class: QueryClass,
}

impl Question {
    pub fn new(name: Name, record_type: QueryType, class: QueryClass) -> Self {
        Self {
            name,
            record_type,
            class,
        }
    }

    pub fn name(&self) -> &Name {
        &self.name
    }

    pub fn record_type(&self) -> QueryType {
        self.record_type
    }

    pub fn class(&self) -> QueryClass {
        self.class
    }
}
