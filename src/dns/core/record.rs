use super::{AData, AaaaData, Class, CnameData, Data, MxData, Name, NsData, RecordType, TxtData};
use std::str::FromStr;

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
            Data::Ns(_) => RecordType::Ns,
            Data::Cname(_) => RecordType::Cname,
            Data::Aaaa(_) => RecordType::Aaaa,
            Data::Mx(_) => RecordType::Mx,
            Data::Txt(_) => RecordType::Txt,
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

    pub fn from_str_relative(s: &str, origin: &Name) -> Result<Self, RecordFromStrErr> {
        let (name_str, s) = s.split_once(' ').ok_or(RecordFromStrErr::Invalid)?;
        let (ttl_str, s) = s.split_once(' ').ok_or(RecordFromStrErr::Invalid)?;
        let (class_str, s) = s.split_once(' ').ok_or(RecordFromStrErr::Invalid)?;
        let (record_type_str, data_str) = s.split_once(' ').ok_or(RecordFromStrErr::Invalid)?;
        let name = Name::from_str_relative(name_str, origin).map_err(RecordFromStrErr::NameErr)?;
        let ttl = ttl_str.parse().map_err(RecordFromStrErr::TtlErr)?;
        let class = Class::from_str(class_str).map_err(RecordFromStrErr::ClassErr)?;
        let record_type =
            RecordType::from_str(record_type_str).map_err(RecordFromStrErr::RecordTypeErr)?;
        let data = match record_type {
            RecordType::A => Data::A(data_str.parse().map_err(RecordFromStrErr::ADataErr)?),
            RecordType::Ns => Data::Ns(
                NsData::from_str_relative(data_str, origin).map_err(RecordFromStrErr::NsDataErr)?,
            ),
            RecordType::Cname => {
                Data::Cname(CnameData::from_str_relative(data_str, &origin).map_err(RecordFromStrErr::CnameDataErr)?)
            }
            RecordType::Aaaa => {
                Data::Aaaa(data_str.parse().map_err(RecordFromStrErr::AaaaDataErr)?)
            }
            RecordType::Mx => Data::Mx(MxData::from_str_relative(data_str, &origin).map_err(RecordFromStrErr::MxDataErr)?),
            RecordType::Txt => Data::Txt(data_str.parse().map_err(RecordFromStrErr::TxtDataErr)?),
            RecordType::Unknown(_) => Err(RecordFromStrErr::Invalid)?,
        };
        Ok(Record::new(name, class, ttl, data))
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RecordFromStrErr {
    Invalid,
    NameErr(<Name as FromStr>::Err),
    TtlErr(<Ttl as FromStr>::Err),
    ClassErr(<Class as FromStr>::Err),
    RecordTypeErr(<RecordType as FromStr>::Err),
    ADataErr(<AData as FromStr>::Err),
    NsDataErr(<NsData as FromStr>::Err),
    CnameDataErr(<CnameData as FromStr>::Err),
    AaaaDataErr(<AaaaData as FromStr>::Err),
    MxDataErr(<MxData as FromStr>::Err),
    TxtDataErr(<TxtData as FromStr>::Err),
}

impl FromStr for Record {
    type Err = RecordFromStrErr;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let (name_str, s) = s.split_once(' ').ok_or(RecordFromStrErr::Invalid)?;
        let (ttl_str, s) = s.split_once(' ').ok_or(RecordFromStrErr::Invalid)?;
        let (class_str, s) = s.split_once(' ').ok_or(RecordFromStrErr::Invalid)?;
        let (record_type_str, data_str) = s.split_once(' ').ok_or(RecordFromStrErr::Invalid)?;
        let name = Name::from_str(name_str).map_err(RecordFromStrErr::NameErr)?;
        let ttl = ttl_str.parse().map_err(RecordFromStrErr::TtlErr)?;
        let class = Class::from_str(class_str).map_err(RecordFromStrErr::ClassErr)?;
        let record_type =
            RecordType::from_str(record_type_str).map_err(RecordFromStrErr::RecordTypeErr)?;
        let data = match record_type {
            RecordType::A => Data::A(data_str.parse().map_err(RecordFromStrErr::ADataErr)?),
            RecordType::Ns => Data::Ns(data_str.parse().map_err(RecordFromStrErr::NsDataErr)?),
            RecordType::Cname => {
                Data::Cname(data_str.parse().map_err(RecordFromStrErr::CnameDataErr)?)
            }
            RecordType::Aaaa => {
                Data::Aaaa(data_str.parse().map_err(RecordFromStrErr::AaaaDataErr)?)
            }
            RecordType::Mx => Data::Mx(data_str.parse().map_err(RecordFromStrErr::MxDataErr)?),
            RecordType::Txt => Data::Txt(data_str.parse().map_err(RecordFromStrErr::TxtDataErr)?),
            RecordType::Unknown(_) => Err(RecordFromStrErr::Invalid)?,
        };
        Ok(Record::new(name, class, ttl, data))
    }
}

impl ToString for Record {
    fn to_string(&self) -> String {
        format!(
            "{} {} {} {} {}",
            self.name.to_string(),
            self.ttl,
            self.class.to_string(),
            self.record_type().to_string(),
            match self.data {
                Data::A(ref data) => data.to_string(),
                Data::Ns(ref data) => data.to_string(),
                Data::Cname(ref data) => data.to_string(),
                Data::Aaaa(ref data) => data.to_string(),
                Data::Mx(ref data) => data.to_string(),
                Data::Txt(ref data) => data.to_string(),
                Data::Unknown(_, ref data) => String::from_utf8_lossy(data).to_string(),
            }
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_valid_record_from_str() {
        // "example.com. 3600 IN A 127.0.0.1"
        let record = Record::from_str("example.com. 3600 IN A 127.0.0.1").unwrap();
        let expected = Record::new(
            "example.com.".parse().unwrap(),
            Class::In,
            3600,
            Data::A("127.0.0.1".parse().unwrap()),
        );
        assert_eq!(record, expected);

        // "example.com. 3600 IN NS ns.example.com."
        let record = Record::from_str("example.com. 3600 IN NS ns.example.com.").unwrap();
        let expected = Record::new(
            "example.com.".parse().unwrap(),
            Class::In,
            3600,
            Data::Ns("ns.example.com.".parse().unwrap()),
        );
        assert_eq!(record, expected);

        // "example.com. 3600 IN CNAME cname.example.com."
        let record = Record::from_str("example.com. 3600 IN CNAME cname.example.com.").unwrap();
        let expected = Record::new(
            "example.com.".parse().unwrap(),
            Class::In,
            3600,
            Data::Cname("cname.example.com.".parse().unwrap()),
        );
        assert_eq!(record, expected);

        // "example.com. 3600 IN AAAA ::1"
        let record = Record::from_str("example.com. 3600 IN AAAA ::1").unwrap();
        let expected = Record::new(
            "example.com.".parse().unwrap(),
            Class::In,
            3600,
            Data::Aaaa("::1".parse().unwrap()),
        );
        assert_eq!(record, expected);

        // "example.com. 3600 IN MX 10 mail.example.com."
        let record = Record::from_str("example.com. 3600 IN MX 10 mail.example.com.").unwrap();
        let expected = Record::new(
            "example.com.".parse().unwrap(),
            Class::In,
            3600,
            Data::Mx(MxData::new(10, "mail.example.com.".parse().unwrap())),
        );
        assert_eq!(record, expected);

        // "example.com. 3600 IN TXT \"Hello, world!\""
        let record = Record::from_str("example.com. 3600 IN TXT \"Hello, world!\"").unwrap();
        let expected = Record::new(
            "example.com.".parse().unwrap(),
            Class::In,
            3600,
            Data::Txt(TxtData::new("Hello, world!".to_string())),
        );
        assert_eq!(record, expected);
    }

    #[test]
    fn test_record_to_string() {
        let record = Record::new(
            "example.com.".parse().unwrap(),
            Class::In,
            3600,
            Data::A("127.0.0.1".parse().unwrap()),
        );
        let expected = "example.com. 3600 IN A 127.0.0.1".to_string();
        assert_eq!(record.to_string(), expected);

        let record = Record::new(
            "example.com.".parse().unwrap(),
            Class::In,
            3600,
            Data::Ns("ns.example.com.".parse().unwrap()),
        );
        let expected = "example.com. 3600 IN NS ns.example.com.".to_string();
        assert_eq!(record.to_string(), expected);

        let record = Record::new(
            "example.com.".parse().unwrap(),
            Class::In,
            3600,
            Data::Cname("cname.example.com.".parse().unwrap()),
        );
        let expected = "example.com. 3600 IN CNAME cname.example.com.".to_string();
        assert_eq!(record.to_string(), expected);
    }

    #[test]
    fn test_record_from_str_relative() {
        let origin = "com.".parse::<Name>().unwrap();

        // "example.com. 3600 IN A 127.0.0.1"
        let record =
            Record::from_str_relative("example.com. 3600 IN A 127.0.0.1", &origin).unwrap();
        let expected = Record::new(
            "example.com.".parse().unwrap(),
            Class::In,
            3600,
            Data::A("127.0.0.1".parse().unwrap()),
        );
        assert_eq!(record, expected);

        // "example 3600 IN A 127.0.0.1"
        let record = Record::from_str_relative("example 3600 IN A 127.0.0.1", &origin).unwrap();
        let expected = Record::new(
            "example.com.".parse().unwrap(),
            Class::In,
            3600,
            Data::A("127.0.0.1".parse().unwrap()),
        );
        assert_eq!(record, expected);

        // "example.com. 3600 IN NS ns.example.com."
        let record =
            Record::from_str_relative("example.com. 3600 IN NS ns.example.com.", &origin).unwrap();
        let expected = Record::new(
            "example.com.".parse().unwrap(),
            Class::In,
            3600,
            Data::Ns("ns.example.com.".parse().unwrap()),
        );
        assert_eq!(record, expected);

        // "example 3600 IN NS ns"
        let record = Record::from_str_relative("example 3600 IN NS ns.example", &origin).unwrap();
        let expected = Record::new(
            "example.com.".parse().unwrap(),
            Class::In,
            3600,
            Data::Ns("ns.example.com.".parse().unwrap()),
        );
        assert_eq!(record, expected);

        // "example.com. 3600 IN CNAME cname.example.com."
        let record = Record::from_str_relative("example.com. 3600 IN CNAME cname.example.com.", &origin).unwrap();
        let expected = Record::new(
            "example.com.".parse().unwrap(),
            Class::In,
            3600,
            Data::Cname("cname.example.com.".parse().unwrap()),
        );
        assert_eq!(record, expected);

        // "example 3600 IN CNAME cname.example"
        let record =
            Record::from_str_relative("example 3600 IN CNAME cname.example", &origin).unwrap();
        let expected = Record::new(
            "example.com.".parse().unwrap(),
            Class::In,
            3600,
            Data::Cname("cname.example.com.".parse().unwrap()),
        );
        assert_eq!(record, expected);

        // "example.com. 3600 IN AAAA ::1"
        let record = Record::from_str_relative("example.com. 3600 IN AAAA ::1", &origin).unwrap();
        let expected = Record::new(
            "example.com.".parse().unwrap(),
            Class::In,
            3600,
            Data::Aaaa("::1".parse().unwrap()),
        );
        assert_eq!(record, expected);

        // "example 3600 IN AAAA ::1"
        let record = Record::from_str_relative("example 3600 IN AAAA ::1", &origin).unwrap();
        let expected = Record::new(
            "example.com.".parse().unwrap(),
            Class::In,
            3600,
            Data::Aaaa("::1".parse().unwrap()),
        );
        assert_eq!(record, expected);

        // "example.com. 3600 IN MX 10 mail.example.com."
        let record = Record::from_str_relative("example.com. 3600 IN MX 10 mail.example.com.", &origin).unwrap();
        let expected = Record::new(
            "example.com.".parse().unwrap(),
            Class::In,
            3600,
            Data::Mx(MxData::new(10, "mail.example.com.".parse().unwrap())),
        );
        assert_eq!(record, expected);

        // "example 3600 IN MX 10 mail.example"
        let record = Record::from_str_relative("example 3600 IN MX 10 mail.example", &origin).unwrap();
        let expected = Record::new(
            "example.com.".parse().unwrap(),
            Class::In,
            3600,
            Data::Mx(MxData::new(10, "mail.example.com.".parse().unwrap())),
        );
        assert_eq!(record, expected);

        // "example.com. 3600 IN TXT \"Hello, world!\""
        let record = Record::from_str_relative("example.com. 3600 IN TXT \"Hello, world!\"", &origin).unwrap();
        let expected = Record::new(
            "example.com.".parse().unwrap(),
            Class::In,
            3600,
            Data::Txt(TxtData::new("Hello, world!".to_string())),
        );
        assert_eq!(record, expected);

        // "example 3600 IN TXT \"Hello, world!\""
        let record = Record::from_str_relative("example 3600 IN TXT \"Hello, world!\"", &origin).unwrap();
        let expected = Record::new(
            "example.com.".parse().unwrap(),
            Class::In,
            3600,
            Data::Txt(TxtData::new("Hello, world!".to_string())),
        );
        assert_eq!(record, expected);
    }
}
