use std::str::FromStr;

use crate::dns::core::Name;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CnameData {
    cname: Name,
}

impl CnameData {
    pub fn new(name: Name) -> Self {
        Self { cname: name }
    }

    pub fn cname(&self) -> &Name {
        &self.cname
    }

    pub fn from_str_relative(s: &str, origin: &Name) -> Result<Self, CnameDataFromStrErr> {
        Ok(Self::new(Name::from_str_relative(s, &origin).map_err(CnameDataFromStrErr::NameFromStrErr)?))
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CnameDataFromStrErr {
    NameFromStrErr(<Name as FromStr>::Err),
}

impl FromStr for CnameData {
    type Err = CnameDataFromStrErr;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(Self::new(s.parse().map_err(Self::Err::NameFromStrErr)?))
    }
}

impl ToString for CnameData {
    fn to_string(&self) -> String {
        self.cname().to_string()
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_from_str() {
        // invalid address
        let data: Result<CnameData, _> = "example .com.".parse();
        assert!(data.is_err());

        // valid address
        let data: CnameData = "example.com.".parse().unwrap();
        assert_eq!(data.cname(), &"example.com.".parse::<Name>().unwrap());
    }

    #[test]
    fn test_to_string() {
        let data = CnameData::new("example.com.".parse().unwrap());
        assert_eq!(data.to_string(), "example.com.");
    }
}
