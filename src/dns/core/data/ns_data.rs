use std::str::FromStr;

use crate::dns::core::Name;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NsData {
    name_server: Name,
}

impl NsData {
    pub fn new(name_server: Name) -> Self {
        Self { name_server }
    }

    pub fn name_server(&self) -> &Name {
        &self.name_server
    }

    pub fn from_str_relative(s: &str, origin: &Name) -> Result<Self, NsDataFromStrErr> {
        Ok(Self::new(Name::from_str_relative(s, &origin).map_err(NsDataFromStrErr::NameFromStrErr)?))
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum NsDataFromStrErr {
    NameFromStrErr(<Name as FromStr>::Err),
}

impl FromStr for NsData {
    type Err = NsDataFromStrErr;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(Self::new(s.parse().map_err(Self::Err::NameFromStrErr)?))
    }


}

impl ToString for NsData {
    fn to_string(&self) -> String {
        self.name_server().to_string()
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_from_str() {
        // invalid address
        let data: Result<NsData, _> = "example .com.".parse();
        assert!(data.is_err());

        // valid address
        let data: NsData = "example.com.".parse().unwrap();
        assert_eq!(data.name_server(), &"example.com.".parse::<Name>().unwrap());
    }

    #[test]
    fn test_to_string() {
        let data = NsData::new("example.com.".parse().unwrap());
        assert_eq!(data.to_string(), "example.com.");
    }
}
