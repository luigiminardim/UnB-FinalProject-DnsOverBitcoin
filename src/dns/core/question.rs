use super::{Name, QueryClass, QueryType, Record};

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

    pub fn matches(&self, record: &Record) -> bool {
        &self.name == record.name()
            && self.query_type.matches(record.record_type())
            && self.query_class.matches(record.class())
    }
}

#[cfg(test)]
mod test {
    use std::net::Ipv4Addr;

    use crate::dns::core::{AData, Class, Data, RecordType};

    use super::*;

    #[test]
    fn test_matches() {
        // matches all
        let question = Question::new(
            "example.com".parse().unwrap(),
            QueryType::Type(RecordType::A),
            QueryClass::Class(Class::IN),
        );
        let record = Record::new(
            "example.com".parse().unwrap(),
            Class::IN,
            0,
            Data::A(AData::new(Ipv4Addr::new(127, 0, 0, 1))),
        );
        assert!(question.matches(&record));

        // matches with ALL type and ANY class
        let question = Question::new(
            "example.com".parse().unwrap(),
            QueryType::All,
            QueryClass::Any,
        );
        let record = Record::new(
            "example.com".parse().unwrap(),
            Class::IN,
            0,
            Data::A(AData::new(Ipv4Addr::new(127, 0, 0, 1))),
        );
        assert!(question.matches(&record));

        // not match name
        let question = Question::new(
            "example.com".parse().unwrap(),
            QueryType::Type(RecordType::A),
            QueryClass::Class(Class::IN),
        );
        let record = Record::new(
            "com".parse().unwrap(),
            Class::IN,
            0,
            Data::A(AData::new(Ipv4Addr::new(127, 0, 0, 1))),
        );
        assert!(!question.matches(&record));

        // not match type
        let question = Question::new(
            "example.com".parse().unwrap(),
            QueryType::Type(RecordType::Cname),
            QueryClass::Class(Class::IN),
        );
        let record = Record::new(
            "example.com".parse().unwrap(),
            Class::IN,
            0,
            Data::A(AData::new(Ipv4Addr::new(127, 0, 0, 1))),
        );
        assert!(!question.matches(&record));
    }
}
