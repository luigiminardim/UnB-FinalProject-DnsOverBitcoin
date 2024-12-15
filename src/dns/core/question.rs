use super::{Name, QueryClass, QueryType};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Question {
    name: Name,
    query_type: QueryType,
    query_class: QueryClass,
}

impl Question {
    pub fn new(name: Name, query_type: QueryType, query_class: QueryClass) -> Self {
        Self {
            name,
            query_type,
            query_class,
        }
    }

    pub fn name(&self) -> &Name {
        &self.name
    }

    pub fn query_type(&self) -> QueryType {
        self.query_type
    }

    pub fn query_class(&self) -> QueryClass {
        self.query_class
    }
}
