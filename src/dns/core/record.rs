use super::{Class, Data, Name, RecordType};

pub type Ttl = u32;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Record {
    name: Name,
    class: Class,
    ttl: Ttl,
    data: Data,
}

impl Record {
    pub fn new(name: Name, class: Class, ttl: u32, data: Data) -> Self {
        Self {
            name,
            class,
            ttl,
            data,
        }
    }

    pub fn record_type(&self) -> RecordType {
        match &self.data {
            Data::A(_) => RecordType::A,
            Data::Ns(_) => RecordType::NS,
            Data::Cname(_) => RecordType::Cname,
            Data::Aaaa(_) => RecordType::Aaaa,
            Data::Unknown(record_type, _) => record_type.clone(),
        }
    }

    pub fn name(&self) -> &Name {
        &self.name
    }

    pub fn class(&self) -> Class {
        self.class.clone()
    }

    pub fn ttl(&self) -> Ttl {
        self.ttl
    }

    pub fn data(&self) -> &Data {
        &self.data
    }
}
