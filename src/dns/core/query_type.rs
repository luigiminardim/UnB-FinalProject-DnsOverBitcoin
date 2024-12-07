use super::record_type::RecordType;

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
    All,
}

impl From<u16> for QueryType {
    fn from(value: u16) -> Self {
        match value {
            252 => QueryType::Axfr,
            253 => QueryType::MailB,
            254 => QueryType::MailA,
            255 => QueryType::All,
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
            QueryType::All => 255,
            QueryType::Type(value) => u16::from(value),
        }
    }
}

impl QueryType {
    pub fn matches(&self, record_type: RecordType) -> bool {
        match self {
            QueryType::All => true,
            QueryType::Type(query_record_type) => query_record_type == &record_type,
            _ => false,
        }
    }
}

#[cfg(test)]
mod test_query_type {
    use super::*;

    #[test]
    fn test_matches() {
        assert!(QueryType::All.matches(RecordType::A));
        assert!(QueryType::Type(RecordType::A).matches(RecordType::A));
        assert!(!QueryType::Type(RecordType::A).matches(RecordType::Unknown(100)));
        assert!(!QueryType::Axfr.matches(RecordType::A));
        assert!(!QueryType::MailB.matches(RecordType::A));
        assert!(!QueryType::MailA.matches(RecordType::A));
    }
}
