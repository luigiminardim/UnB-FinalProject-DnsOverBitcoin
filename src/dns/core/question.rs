use std::str::FromStr;

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

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum QuestionFromStrErr {
    Invalid,
    NameFromStrErr(<Name as FromStr>::Err),
    QueryTypeFromStrErr(<QueryType as FromStr>::Err),
    QueryClassFromStrErr(<QueryClass as FromStr>::Err),
}

impl FromStr for Question {
    type Err = QuestionFromStrErr;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut parts = s.split_whitespace();
        let name = parts
            .next()
            .ok_or(QuestionFromStrErr::Invalid)?
            .parse()
            .map_err(QuestionFromStrErr::NameFromStrErr)?;
        let query_class = parts
            .next()
            .ok_or(QuestionFromStrErr::Invalid)?
            .parse()
            .map_err(QuestionFromStrErr::QueryClassFromStrErr)?;
        let query_type = parts
            .next()
            .ok_or(QuestionFromStrErr::Invalid)?
            .parse()
            .map_err(QuestionFromStrErr::QueryTypeFromStrErr)?;
        Ok(Self::new(name, query_type, query_class))
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
            "example.com.".parse().unwrap(),
            QueryType::Type(RecordType::A),
            QueryClass::Class(Class::In),
        );
        let record = Record::new(
            "example.com.".parse().unwrap(),
            Class::In,
            0,
            Data::A(AData::new(Ipv4Addr::new(127, 0, 0, 1))),
        );
        assert!(question.matches(&record));

        // matches with ANY type and ANY class
        let question = Question::new(
            "example.com.".parse().unwrap(),
            QueryType::Any,
            QueryClass::Any,
        );
        let record = Record::new(
            "example.com.".parse().unwrap(),
            Class::In,
            0,
            Data::A(AData::new(Ipv4Addr::new(127, 0, 0, 1))),
        );
        assert!(question.matches(&record));

        // not match name
        let question = Question::new(
            "example.com.".parse().unwrap(),
            QueryType::Type(RecordType::A),
            QueryClass::Class(Class::In),
        );
        let record = Record::new(
            "com.".parse().unwrap(),
            Class::In,
            0,
            Data::A(AData::new(Ipv4Addr::new(127, 0, 0, 1))),
        );
        assert!(!question.matches(&record));

        // not match type
        let question = Question::new(
            "example.com.".parse().unwrap(),
            QueryType::Type(RecordType::Cname),
            QueryClass::Class(Class::In),
        );
        let record = Record::new(
            "example.com.".parse().unwrap(),
            Class::In,
            0,
            Data::A(AData::new(Ipv4Addr::new(127, 0, 0, 1))),
        );
        assert!(!question.matches(&record));
    }

    #[test]
    fn test_from_str() {
        // valid "example.com IN A"
        let question = "example.com. IN A".parse::<Question>().unwrap();
        assert_eq!(
            question,
            Question::new(
                "example.com.".parse().unwrap(),
                QueryType::Type(RecordType::A),
                QueryClass::Class(Class::In)
            )
        );

        // valid "example.com IN ANY"
        let question = "example.com. IN *".parse::<Question>().unwrap();
        assert_eq!(
            question,
            Question::new(
                "example.com.".parse().unwrap(),
                QueryType::Any,
                QueryClass::Class(Class::In)
            )
        );

        // invalid "example.com IN"
        assert!("example.com IN".parse::<Question>().is_err());

        // invalid "example.com"
        assert!("example.com".parse::<Question>().is_err());

        // invalid "example .com IN A"
        assert!("example .com IN A".parse::<Question>().is_err());
    }
}
