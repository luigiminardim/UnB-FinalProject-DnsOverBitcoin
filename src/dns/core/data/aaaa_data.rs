use std::{net::Ipv6Addr, str::FromStr};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AaaaData {
    address: Ipv6Addr,
}

impl AaaaData {
    pub fn new(address: Ipv6Addr) -> Self {
        Self { address }
    }

    pub fn address(&self) -> Ipv6Addr {
        self.address
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AaaaDataFromStrErr {
    AddrParseError(std::net::AddrParseError),
}

impl FromStr for AaaaData {
    type Err = AaaaDataFromStrErr;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(Self::new(
            s.parse().map_err(AaaaDataFromStrErr::AddrParseError)?,
        ))
    }
}

impl ToString for AaaaData {
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
        let data: Result<AaaaData, _> = "127.0.0.256".parse();
        assert!(data.is_err());

        // valid address
        let data: AaaaData = "2001:0db8:85a3:0000:0000:8a2e:0370:7334".parse().unwrap();
        assert_eq!(
            data.address(),
            Ipv6Addr::new(0x2001, 0x0db8, 0x85a3, 0x0000, 0x0000, 0x8a2e, 0x0370, 0x7334)
        );
    }

    #[test]
    fn test_to_string() {
        let data = AaaaData::new(Ipv6Addr::new(
            0x2001, 0x0db8, 0x85a3, 0x0000, 0x0000, 0x8a2e, 0x0370, 0x7334,
        ));
        assert_eq!(data.to_string(), "2001:db8:85a3::8a2e:370:7334");
    }
}
