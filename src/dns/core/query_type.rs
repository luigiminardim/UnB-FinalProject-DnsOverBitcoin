use std::str::FromStr;

use super::RecordType;

/// QTYPE fields appear in the question part of a query. QTYPES are a superset
/// of TYPEs, hence all TYPEs are valid QTYPEs.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum QueryType {
    /// Represents a regular RecordType
    Type(RecordType),

    /// Request for a transfer of an entire zone
    Axfr,

    /// Request for mailbox-related records (MB, MG or MR)
    MailB,

    /// Request for mail agent RRs (Obsolete - see MX)
    MailA,

    /// Request for all records (*)
    Any,
}

impl From<u16> for QueryType {
    fn from(value: u16) -> Self {
        match value {
            252 => QueryType::Axfr,
            253 => QueryType::MailB,
            254 => QueryType::MailA,
            255 => QueryType::Any,
            _ => QueryType::Type(RecordType::from(value)),
        }
    }
}

impl From<QueryType> for u16 {
    fn from(value: QueryType) -> Self {
        match value {
            QueryType::Axfr => 252,
            QueryType::MailB => 253,
            QueryType::MailA => 254,
            QueryType::Any => 255,
            QueryType::Type(value) => u16::from(value),
        }
    }
}

impl QueryType {
    pub fn matches(&self, record_type: RecordType) -> bool {
        match self {
            QueryType::Any => true,
            QueryType::Type(query_record_type) => query_record_type == &record_type,
            _ => false,
        }
    }
}

impl FromStr for QueryType {
    type Err = <RecordType as FromStr>::Err;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "AXFR" => Ok(QueryType::Axfr),
            "MAILB" => Ok(QueryType::MailB),
            "MAILA" => Ok(QueryType::MailA),
            "ANY" => Ok(QueryType::Any),
            _ => RecordType::from_str(s).map(QueryType::Type),
        }
    }
}

impl ToString for QueryType {
    fn to_string(&self) -> String {
        match self {
            QueryType::Type(record_type) => record_type.to_string(),
            QueryType::Axfr => "AXFR".to_string(),
            QueryType::MailB => "MAILB".to_string(),
            QueryType::MailA => "MAILA".to_string(),
            QueryType::Any => "ANY".to_string(),
        }
    }
}

#[cfg(test)]
mod test_query_type {
    use super::*;

    #[test]
    fn test_matches() {
        assert!(QueryType::Any.matches(RecordType::A));
        assert!(QueryType::Type(RecordType::A).matches(RecordType::A));
        assert!(!QueryType::Type(RecordType::A).matches(RecordType::Unknown(100)));
        assert!(!QueryType::Axfr.matches(RecordType::A));
        assert!(!QueryType::MailB.matches(RecordType::A));
        assert!(!QueryType::MailA.matches(RecordType::A));
    }

    #[test]
    fn test_from_str() {
        assert_eq!(QueryType::from_str("AXFR").unwrap(), QueryType::Axfr);
        assert_eq!(QueryType::from_str("MAILB").unwrap(), QueryType::MailB);
        assert_eq!(QueryType::from_str("MAILA").unwrap(), QueryType::MailA);
        assert_eq!(QueryType::from_str("ANY").unwrap(), QueryType::Any);

        assert_eq!(
            QueryType::from_str("A").unwrap(),
            QueryType::Type(RecordType::A)
        );
        assert_eq!(
            QueryType::from_str("AAAA").unwrap(),
            QueryType::Type(RecordType::Aaaa)
        );
        assert_eq!(
            QueryType::from_str("CNAME").unwrap(),
            QueryType::Type(RecordType::Cname)
        );
        assert_eq!(
            QueryType::from_str("MX").unwrap(),
            QueryType::Type(RecordType::Mx)
        );
        assert_eq!(
            QueryType::from_str("NS").unwrap(),
            QueryType::Type(RecordType::Ns)
        );
        assert_eq!(
            QueryType::from_str("TXT").unwrap(),
            QueryType::Type(RecordType::Txt)
        );
    }

    #[test]
    fn test_to_string() {
        assert_eq!(QueryType::Axfr.to_string(), "AXFR");
        assert_eq!(QueryType::MailB.to_string(), "MAILB");
        assert_eq!(QueryType::MailA.to_string(), "MAILA");
        assert_eq!(QueryType::Any.to_string(), "ANY");

        assert_eq!(QueryType::Type(RecordType::A).to_string(), "A");
        assert_eq!(QueryType::Type(RecordType::Aaaa).to_string(), "AAAA");
        assert_eq!(QueryType::Type(RecordType::Cname).to_string(), "CNAME");
        assert_eq!(QueryType::Type(RecordType::Mx).to_string(), "MX");
        assert_eq!(QueryType::Type(RecordType::Ns).to_string(), "NS");
        assert_eq!(QueryType::Type(RecordType::Txt).to_string(), "TXT");
    }
}
