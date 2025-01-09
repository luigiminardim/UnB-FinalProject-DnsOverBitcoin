use std::vec;

use super::Resolver;
use crate::dns::core::{
    Data, Message, Name, OpCode, QueryClass, QueryType, Question, Record, RecordType, ResponseCode,
};
use async_trait::async_trait;

pub struct InMemoryAuthority {
    records: Vec<Record>,
}

impl InMemoryAuthority {
    pub fn new(records: Vec<Record>) -> Self {
        InMemoryAuthority { records }
    }

    fn resolve_query(&self, request: &Message) -> Message {
        let question = match request.questions().first() {
            None => return request.into_response(ResponseCode::FormatError),
            Some(question) => question,
        };
        let (answer_records, authority_records, additional_records) =
            self.resolve_question(&question);
        request
            .into_response(ResponseCode::NoError)
            .set_answers(answer_records)
            .set_authorities(authority_records)
            .set_additional(additional_records)
    }

    fn lookup(&self, question: &Question) -> Vec<Record> {
        self.records
            .iter()
            .filter(|record| question.matches(record))
            .cloned()
            .collect()
    }

    fn resolve_question(&self, question: &Question) -> (Vec<Record>, Vec<Record>, Vec<Record>) {
        // Find answer records;
        let answer_records = self.lookup(question);
        if !answer_records.is_empty() || question.query_type() == QueryType::Type(RecordType::Cname)
        {
            return (answer_records, vec![], vec![]);
        }

        // Find CNAME answer records;
        let cname_record = self
            .lookup(&Question::new(
                question.name().clone(),
                QueryType::Type(RecordType::Cname),
                question.query_class(),
            ))
            .first()
            .cloned();
        if let Some(cname_record) = cname_record {
            let question_name = match cname_record.data() {
                Data::Cname(cname_data) => cname_data.cname().clone(),
                _ => return (vec![], vec![], vec![]), // unreachable
            };
            let (mut answer, authority, additional) = self.resolve_question(&Question::new(
                question_name,
                question.query_type(),
                question.query_class(),
            ));
            answer.insert(0, cname_record);
            return (answer, authority, additional);
        }

        // Find authoritative referral records;
        let authority_records = self.find_authority_records(question);
        if !authority_records.is_empty() {
            // Find glue records for the authority records;
            let glue_records = self.find_authority_glue_records(&authority_records);
            return (vec![], authority_records, glue_records);
        }

        return (vec![], vec![], vec![]);
    }

    fn find_authority_records(&self, question: &Question) -> Vec<Record> {
        let resource_nodes = question
            .name()
            .suffix_names()
            .iter()
            .rev()
            .cloned()
            .collect::<Vec<Name>>();

        for name in resource_nodes {
            let authority_records = self.lookup(&Question::new(
                name.clone(),
                QueryType::Type(RecordType::NS),
                question.query_class(),
            ));
            if !authority_records.is_empty() {
                return authority_records;
            }
        }
        vec![]
    }

    fn find_authority_glue_records(&self, authority_records: &Vec<Record>) -> Vec<Record> {
        let mut records_to_glue = authority_records.clone();
        let mut glue_records = vec![];
        while let Some(record_to_glue) = records_to_glue.pop() {
            let name = match record_to_glue.data() {
                Data::Ns(ns_data) => ns_data.name_server(),
                Data::Cname(cname_data) => cname_data.cname(),
                _ => continue,
            };
            let a_records = self.lookup(&Question::new(
                name.clone(),
                QueryType::Type(RecordType::A),
                QueryClass::Class(record_to_glue.class()),
            ));
            let aaaa_records = self.lookup(&Question::new(
                name.clone(),
                QueryType::Type(RecordType::Aaaa),
                QueryClass::Class(record_to_glue.class()),
            ));
            let cname_records = self.lookup(&Question::new(
                name.clone(),
                QueryType::Type(RecordType::Cname),
                QueryClass::Class(record_to_glue.class()),
            ));
            glue_records.extend(a_records.clone());
            glue_records.extend(aaaa_records.clone());
            glue_records.extend(cname_records.clone());
            records_to_glue.extend(a_records);
            records_to_glue.extend(aaaa_records);
            records_to_glue.extend(cname_records);
        }
        glue_records
    }
}

#[async_trait]
impl Resolver for InMemoryAuthority {
    async fn resolve(&self, request: &Message) -> Message {
        match request.opcode() {
            OpCode::Query => self.resolve_query(request),
            _ => request.into_response(ResponseCode::NotImplemented),
        }
    }
}

#[cfg(test)]
mod test {
    use std::net::Ipv4Addr;

    use crate::dns::core::{AData, Class, CnameData, MxData, NsData};

    use super::*;

    fn query_message_from_question(question: Question) -> Message {
        Message::new(0)
            .set_opcode(OpCode::Query)
            .set_questions(vec![question])
    }

    #[tokio::test]
    async fn test_resolve_aname() {
        let records = vec![
            Record::new(
                "example.com".parse().unwrap(),
                Class::IN,
                0,
                Data::A(AData::new(Ipv4Addr::new(127, 0, 0, 1))),
            ),
            Record::new(
                "example.com".parse().unwrap(),
                Class::IN,
                0,
                Data::A(AData::new(Ipv4Addr::new(127, 0, 0, 2))),
            ),
            Record::new(
                "not-example.com".parse().unwrap(),
                Class::IN,
                0,
                Data::A(AData::new(Ipv4Addr::new(127, 0, 0, 3))),
            ),
        ];
        let question = Question::new(
            "example.com".parse().unwrap(),
            QueryType::Type(RecordType::A),
            QueryClass::Class(Class::IN),
        );
        let request = query_message_from_question(question);
        let response = InMemoryAuthority::new(records).resolve(&request).await;
        assert_eq!(response.answers().len(), 2);
        response.answers().iter().for_each(|record| {
            assert_eq!(record.name(), &"example.com".parse().unwrap());
            assert_eq!(record.class(), Class::IN);
            assert_eq!(record.ttl(), 0);
            assert!(matches!(record.data(), Data::A(_)));
        });
    }

    #[tokio::test]
    async fn test_resolve_aname_with_cname() {
        let cname_record = Record::new(
            "example.com".parse().unwrap(),
            Class::IN,
            0,
            Data::Cname(CnameData::new("a.example.com".parse().unwrap())),
        );
        let a_record = Record::new(
            "a.example.com".parse().unwrap(),
            Class::IN,
            0,
            Data::A(AData::new(Ipv4Addr::new(127, 0, 0, 1))),
        );
        let records = vec![cname_record.clone(), a_record.clone()];
        let question = Question::new(
            "example.com".parse().unwrap(),
            QueryType::Type(RecordType::A),
            QueryClass::Class(Class::IN),
        );
        let request = query_message_from_question(question);
        let response = InMemoryAuthority::new(records).resolve(&request).await;
        assert_eq!(response.answers().len(), 2);
        assert!(response
            .answers()
            .iter()
            .any(|record| record == &cname_record));
        response.answers().iter().any(|record| record == &a_record);
    }

    /// C.ISI.EDU name server https://datatracker.ietf.org/doc/html/rfc1034#section-6.1
    fn c_isi_edu_zone() -> InMemoryAuthority {
        let records = vec![
            Record::new(
                "C.ISI.EDU.".parse().unwrap(),
                Class::IN,
                604800,
                Data::Ns(NsData::new("A.ISI.EDU.".parse().unwrap())),
            ),
            Record::new(
                "C.ISI.EDU.".parse().unwrap(),
                Class::IN,
                604800,
                Data::Ns(NsData::new("C.ISI.EDU.".parse().unwrap())),
            ),
            Record::new(
                "C.ISI.EDU.".parse().unwrap(),
                Class::IN,
                604800,
                Data::Ns(NsData::new("C.ISI.EDU.".parse().unwrap())),
            ),
            Record::new(
                "C.ISI.EDU.".parse().unwrap(),
                Class::IN,
                604800,
                Data::Ns(NsData::new("SRI-NIC.ARPA.".parse().unwrap())),
            ),
            Record::new(
                "MIL.".parse().unwrap(),
                Class::IN,
                86400,
                Data::Ns(NsData::new("SRI-NIC.ARPA.".parse().unwrap())),
            ),
            Record::new(
                "MIL.".parse().unwrap(),
                Class::IN,
                86400,
                Data::Ns(NsData::new("A.ISI.EDU.".parse().unwrap())),
            ),
            Record::new(
                "EDU.".parse().unwrap(),
                Class::IN,
                86400,
                Data::Ns(NsData::new("SRI-NIC.ARPA.".parse().unwrap())),
            ),
            Record::new(
                "EDU.".parse().unwrap(),
                Class::IN,
                86400,
                Data::Ns(NsData::new("C.ISI.EDU.".parse().unwrap())),
            ),
            Record::new(
                "SRI-NIC.ARPA.".parse().unwrap(),
                Class::IN,
                86400,
                Data::A(AData::new("26.0.0.73".parse().unwrap())),
            ),
            Record::new(
                "SRI-NIC.ARPA.".parse().unwrap(),
                Class::IN,
                86400,
                Data::A(AData::new("10.0.0.51".parse().unwrap())),
            ),
            Record::new(
                "SRI-NIC.ARPA.".parse().unwrap(),
                Class::IN,
                86400,
                Data::Mx(MxData::new(0, "SRI-NIC.ARPA.".parse().unwrap())),
            ),
            Record::new(
                "USC-ISIC.ARPA.".parse().unwrap(),
                Class::IN,
                86400,
                Data::Cname(CnameData::new("C.ISI.EDU.".parse().unwrap())),
            ),
            Record::new(
                "A.ISI.EDU.".parse().unwrap(),
                Class::IN,
                86400,
                Data::A(AData::new("26.3.0.103".parse().unwrap())),
            ),
            Record::new(
                "C.ISI.EDU.".parse().unwrap(),
                Class::IN,
                86400,
                Data::A(AData::new("10.0.0.52".parse().unwrap())),
            ),
        ];
        InMemoryAuthority::new(records)
    }

    /// QNAME=SRI-NIC.ARPA, QTYPE=A https://datatracker.ietf.org/doc/html/rfc1034#section-6.2.1
    #[tokio::test]
    async fn test_section_6_2_1() {
        let authority = c_isi_edu_zone();
        let request = query_message_from_question(Question::new(
            "SRI-NIC.ARPA.".parse().unwrap(),
            QueryType::Type(RecordType::A),
            QueryClass::Class(Class::IN),
        ));
        let response = authority.resolve(&request).await;
        assert_eq!(response.answers().len(), 2);
        let expected_answers = vec![
            Record::new(
                "SRI-NIC.ARPA.".parse().unwrap(),
                Class::IN,
                86400,
                Data::A(AData::new("26.0.0.73".parse().unwrap())),
            ),
            Record::new(
                "SRI-NIC.ARPA.".parse().unwrap(),
                Class::IN,
                86400,
                Data::A(AData::new("10.0.0.51".parse().unwrap())),
            ),
        ];
        response.answers().iter().for_each(|record| {
            assert!(expected_answers.contains(record));
        });
    }

    /// QNAME=SRI-NIC.ARPA, QTYPE=* https://datatracker.ietf.org/doc/html/rfc1034#section-6.2.2
    #[tokio::test]
    async fn test_section_6_2_2() {
        let authority = c_isi_edu_zone();
        let request = query_message_from_question(Question::new(
            "SRI-NIC.ARPA.".parse().unwrap(),
            QueryType::All,
            QueryClass::Class(Class::IN),
        ));
        let response = authority.resolve(&request).await;
        assert_eq!(response.answers().len(), 3);
        let expected_answers = vec![
            Record::new(
                "SRI-NIC.ARPA.".parse().unwrap(),
                Class::IN,
                86400,
                Data::A(AData::new("26.0.0.73".parse().unwrap())),
            ),
            Record::new(
                "SRI-NIC.ARPA.".parse().unwrap(),
                Class::IN,
                86400,
                Data::A(AData::new("10.0.0.51".parse().unwrap())),
            ),
            Record::new(
                "SRI-NIC.ARPA.".parse().unwrap(),
                Class::IN,
                86400,
                Data::Mx(MxData::new(0, "SRI-NIC.ARPA.".parse().unwrap())),
            ),
        ];
        response.answers().iter().for_each(|record| {
            assert!(expected_answers.contains(record));
        });
    }

    /// QNAME=SRI-NIC.ARPA, QTYPE=MX https://datatracker.ietf.org/doc/html/rfc1034#section-6.2.3
    #[tokio::test]
    async fn test_section_6_2_3() {
        let authority = c_isi_edu_zone();
        let request = query_message_from_question(Question::new(
            "SRI-NIC.ARPA.".parse().unwrap(),
            QueryType::Type(RecordType::Mx),
            QueryClass::Class(Class::IN),
        ));
        let response = authority.resolve(&request).await;
        assert_eq!(response.answers().len(), 1);
        let expected_answers = vec![Record::new(
            "SRI-NIC.ARPA.".parse().unwrap(),
            Class::IN,
            86400,
            Data::Mx(MxData::new(0, "sri-nic.arpa".parse().unwrap())),
        )];
        response.answers().iter().for_each(|record| {
            assert!(expected_answers.contains(record));
        });
    }

    /// QNAME=SRI-NIC.ARPA, QTYPE=NS https://datatracker.ietf.org/doc/html/rfc1034#section-6.2.4
    #[tokio::test]
    async fn test_section_6_2_4() {
        let authority = c_isi_edu_zone();
        let request = query_message_from_question(Question::new(
            "SRI-NIC.ARPA.".parse().unwrap(),
            QueryType::Type(RecordType::NS),
            QueryClass::Class(Class::IN),
        ));
        let response = authority.resolve(&request).await;
        assert_eq!(response.answers().len(), 0);
        let expected_answers = vec![];
        response.answers().iter().for_each(|record| {
            assert!(expected_answers.contains(record));
        });
    }

    /// QNAME=BRL.MIL, QTYPE=A https://datatracker.ietf.org/doc/html/rfc1034#section-6.2.6
    #[tokio::test]
    async fn test_section_6_2_6() {
        let authority = c_isi_edu_zone();
        let response = authority
            .resolve(&query_message_from_question(Question::new(
                "BRL.MIL.".parse().unwrap(),
                QueryType::Type(RecordType::A),
                QueryClass::Class(Class::IN),
            )))
            .await;
        assert_eq!(response.answers().len(), 0);
        assert_eq!(response.authorities().len(), 2);
        let expected_authorities = vec![
            Record::new(
                "MIL.".parse().unwrap(),
                Class::IN,
                86400,
                Data::Ns(NsData::new("SRI-NIC.ARPA.".parse().unwrap())),
            ),
            Record::new(
                "MIL.".parse().unwrap(),
                Class::IN,
                86400,
                Data::Ns(NsData::new("A.ISI.EDU.".parse().unwrap())),
            ),
        ];
        response.answers().iter().for_each(|record| {
            assert!(expected_authorities.contains(record));
        });
        assert_eq!(response.additional().len(), 3);
        let expected_additional = vec![
            Record::new(
                "A.ISI.EDU.".parse().unwrap(),
                Class::IN,
                86400,
                Data::A(AData::new("26.3.0.103".parse().unwrap())),
            ),
            Record::new(
                "SRI-NIC.ARPA.".parse().unwrap(),
                Class::IN,
                86400,
                Data::A(AData::new("26.0.0.73".parse().unwrap())),
            ),
            Record::new(
                "SRI-NIC.ARPA.".parse().unwrap(),
                Class::IN,
                86400,
                Data::A(AData::new("10.0.0.51".parse().unwrap())),
            ),
        ];
        response.additional().iter().for_each(|record| {
            assert!(expected_additional.contains(record));
        });
    }

    /// QNAME=USC-ISIC.ARPA, QTYPE=A https://datatracker.ietf.org/doc/html/rfc1034#section-6.2.7
    #[tokio::test]
    async fn test_section_6_2_7() {
        let authority = c_isi_edu_zone();
        let request = query_message_from_question(Question::new(
            "USC-ISIC.ARPA".parse().unwrap(),
            QueryType::Type(RecordType::A),
            QueryClass::Class(Class::IN),
        ));
        let response = authority.resolve(&request).await;
        assert_eq!(response.answers().len(), 2);
        let expected_answers = vec![
            Record::new(
                "USC-ISIC.ARPA.".parse().unwrap(),
                Class::IN,
                86400,
                Data::Cname(CnameData::new("C.ISI.EDU.".parse().unwrap())),
            ),
            Record::new(
                "C.ISI.EDU.".parse().unwrap(),
                Class::IN,
                86400,
                Data::A(AData::new("10.0.0.52".parse().unwrap())),
            ),
        ];
        response.answers().iter().for_each(|record| {
            assert!(expected_answers.contains(record));
        });
    }

    /// QNAME=USC-ISIC.ARPA, QTYPE=CNAME https://datatracker.ietf.org/doc/html/rfc1034#section-6.2.8
    #[tokio::test]
    async fn test_section_6_2_8() {
        let authority = c_isi_edu_zone();
        let request = query_message_from_question(Question::new(
            "USC-ISIC.ARPA.".parse().unwrap(),
            QueryType::Type(RecordType::Cname),
            QueryClass::Class(Class::IN),
        ));
        let response = authority.resolve(&request).await;
        assert_eq!(response.answers().len(), 1);
        let expected_answers = vec![Record::new(
            "USC-ISIC.ARPA.".parse().unwrap(),
            Class::IN,
            86400,
            Data::Cname(CnameData::new("C.ISI.EDU.".parse().unwrap())),
        )];
        response.answers().iter().for_each(|record| {
            assert!(expected_answers.contains(record));
        });
    }
}
