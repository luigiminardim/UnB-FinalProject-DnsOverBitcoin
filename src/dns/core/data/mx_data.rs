use std::str::FromStr;

use crate::dns::core::Name;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MxData {
    /// A 16 bit integer which specifies the preference given to this RR among
    /// others at the same owner.  Lower values are preferred.
    preference: u16,

    /// A <domain-name> which specifies a host willing to act as a mail
    /// exchange for the owner name.
    exchange: Name,
}

impl MxData {
    pub fn new(preference: u16, exchange: Name) -> Self {
        Self {
            preference,
            exchange,
        }
    }

    pub fn preference(&self) -> u16 {
        self.preference
    }

    pub fn exchange(&self) -> &Name {
        &self.exchange
    }

    pub fn from_str_relative(s: &str, origin: &Name) -> Result<Self, MxDataFromStrErr> {
        let (preference_str, exchange_str) =
            s.split_once(' ').ok_or(MxDataFromStrErr::InvalidFormat)?;
        let preference = preference_str
            .parse()
            .map_err(MxDataFromStrErr::PreferenceFromStrErr)?;
        let exchange = Name::from_str_relative(exchange_str, origin)
            .map_err(MxDataFromStrErr::ExchangeFromStrErr)?;
        Ok(Self::new(preference, exchange))
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum MxDataFromStrErr {
    InvalidFormat,
    PreferenceFromStrErr(std::num::ParseIntError),
    ExchangeFromStrErr(<Name as FromStr>::Err),
}

impl FromStr for MxData {
    type Err = MxDataFromStrErr;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let (preference_str, exchange_str) =
            s.split_once(' ').ok_or(MxDataFromStrErr::InvalidFormat)?;
        let preference = preference_str
            .parse()
            .map_err(MxDataFromStrErr::PreferenceFromStrErr)?;
        let exchange = exchange_str
            .parse()
            .map_err(MxDataFromStrErr::ExchangeFromStrErr)?;
        Ok(Self::new(preference, exchange))
    }
}

impl ToString for MxData {
    fn to_string(&self) -> String {
        format!("{} {}", self.preference, self.exchange.to_string())
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_from_str() {
        // invalid exchange
        let data: Result<MxData, _> = "0 example .com.".parse();
        assert!(data.is_err());

        // invalid preference
        let data: Result<MxData, _> = "preference example.com.".parse();
        assert!(data.is_err());

        // valid mx data
        let data: MxData = "0 example.com.".parse().unwrap();
        assert_eq!(data.preference(), 0);
        assert_eq!(data.exchange(), &"example.com.".parse::<Name>().unwrap());
    }

    #[test]
    fn test_to_string() {
        let data = MxData::new(0, "example.com.".parse().unwrap());
        assert_eq!(data.to_string(), "0 example.com.");
    }
}
