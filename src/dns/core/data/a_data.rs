use std::{net::Ipv4Addr, str::FromStr};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AData {
    address: Ipv4Addr,
}

impl AData {
    pub fn new(address: Ipv4Addr) -> Self {
        Self { address }
    }

    pub fn address(&self) -> Ipv4Addr {
        self.address
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ADataFromStrErr {
    AddrParseError(std::net::AddrParseError),
}

impl FromStr for AData {
    type Err = ADataFromStrErr;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(Self::new(
            s.parse().map_err(ADataFromStrErr::AddrParseError)?,
        ))
    }
}

impl ToString for AData {
    fn to_string(&self) -> String {
        self.address.to_string()
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_from_str() {
        // invalid address
        let data: Result<AData, _> = "127.0.0.256".parse();
        assert!(data.is_err());

        // valid address
        let data: AData = "127.0.0.1".parse().unwrap();
        assert_eq!(data.address(), Ipv4Addr::new(127, 0, 0, 1));
    }

    #[test]
    fn test_to_string() {
        let data = AData::new(Ipv4Addr::new(127, 0, 0, 1));
        assert_eq!(data.to_string(), "127.0.0.1");
    }
}
