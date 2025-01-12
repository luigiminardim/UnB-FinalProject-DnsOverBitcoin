use std::str::FromStr;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TxtData {
    text: String,
}

impl TxtData {
    pub fn new(text: String) -> Self {
        Self { text }
    }

    pub fn text(&self) -> &str {
        &self.text
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TxtDataFromStrErr {
    Invalid,
}

impl FromStr for TxtData {
    type Err = TxtDataFromStrErr;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if s.len() < 2 || s.chars().next() != Some('"') || s.chars().last() != Some('"') {
            return Err(TxtDataFromStrErr::Invalid);
        }
        let text = s[1..s.len() - 1].to_string();
        Ok(TxtData::new(text))
    }
}

impl ToString for TxtData {
    fn to_string(&self) -> String {
        format!("\"{}\"", self.text)
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_from_str() {
        // valid address
        let data: TxtData = "\"abc 123\"".parse().unwrap();
        assert_eq!(data.text(), "abc 123");
    }

    #[test]
    fn test_to_string() {
        let data = TxtData::new("abc 123".to_string());
        assert_eq!(data.to_string(), "\"abc 123\"");
    }
}
