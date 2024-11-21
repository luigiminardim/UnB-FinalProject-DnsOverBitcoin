use std::str::FromStr;

use super::label::{FromStrErr as LabelFromStrErr, Label};

const MAX_NAME_LEN: usize = 255;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Name(Vec<Label>);

/// Represents a domain name definition error.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum NameErr {
    /// A domain name must not have an empty label
    EmptyLabel,

    /// The total number of octets that represent a domain name (i.e., the sum
    /// of all label octets and label lengths) is limited to 255.
    LenLimit,
}

/// represents a complete domain name (often called "absolute"). For example,
/// "poneria.ISI.EDU.". Since a complete domain name ends with the root label,
/// this leads to a printed form which ends in a dot.
impl Name {
    /// Create a domain name from a list of labels without the null label at the end.
    /// Don't use null labels
    pub fn create(labels: Vec<Label>) -> Result<Name, NameErr> {
        if labels.iter().any(Label::is_null) {
            return Err(NameErr::EmptyLabel);
        }
        let sum_label_octets_len = labels.iter().map(Label::len).sum::<usize>();
        let num_label_len = labels.len() + 1; // add 1 for the null label
        if sum_label_octets_len + num_label_len > MAX_NAME_LEN {
            return Err(NameErr::LenLimit);
        }
        Ok(Name(labels))
    }

    /// Return the root (".") domain name
    pub fn root() -> Self {
        Name::create(vec![]).unwrap()
    }

    /// Check if it's the "." domain name.
    pub fn is_root(&self) -> bool {
        self.0.len() == 0
    }

    /// Check if it's a subdomain of the given domain. "example.com." is a
    /// subdomain of "com.".
    pub fn is_subdomain(&self, domain: &Name) -> bool {
        let num_sub_labels = self.0.len();
        let num_domain_labels = domain.0.len();
        if num_sub_labels <= num_domain_labels {
            return false;
        }
        let subdomain_labels = &self.0[num_sub_labels - num_domain_labels..];
        subdomain_labels == domain.0.as_slice()
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum FromStrErr {
    NameErr(NameErr),
    LabelFromStrErr(LabelFromStrErr),
}

impl FromStr for Name {
    type Err = FromStrErr;

    fn from_str(str: &str) -> Result<Self, Self::Err> {
        // remove the trailing dot if it exists
        let str = match str.chars().last() {
            Some('.') => &str[..str.len() - 1],
            _ => str,
        };
        if let "" = str {
            return Ok(Name::root());
        }
        let labels = str
            .split('.')
            .map(|label_str| label_str.parse::<Label>())
            .collect::<Result<Vec<Label>, LabelFromStrErr>>()
            .map_err(|err| FromStrErr::LabelFromStrErr(err))?;
        Name::create(labels).map_err(|err| FromStrErr::NameErr(err))
    }
}

impl ToString for Name {
    fn to_string(&self) -> String {
        let mut str = self
            .0
            .iter()
            .map(|label| label.to_string())
            .collect::<Vec<String>>()
            .join(".");
        str.push('.');
        str
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create() {
        // empty domain name is the root domain name
        let name: Result<Name, NameErr> = Name::create(vec![]);
        assert!(name.is_ok());

        // domain name with a single null label is invalid
        let name = Name::create(vec![Label::null()]);
        assert_eq!(name.unwrap_err(), NameErr::EmptyLabel);

        // domain name with a single non-null label is valid
        let name = Name::create(vec!["example".parse().unwrap()]);
        assert!(name.is_ok());

        // domain name with a single non-null label followed by a null label is invalid
        let name = Name::create(vec!["example".parse().unwrap(), Label::null()]);
        assert_eq!(name.unwrap_err(), NameErr::EmptyLabel);

        // domain name with a null label in the middle is invalid
        let name: Result<Name, NameErr> = Name::create(vec![
            "example".parse().unwrap(),
            Label::null(),
            "com".parse().unwrap(),
        ]);
        assert_eq!(name.unwrap_err(), NameErr::EmptyLabel);

        // domain name with a null label at the start is invalid
        let name: Result<Name, NameErr> = Name::create(vec![
            Label::null(),
            "example".parse().unwrap(),
            "com".parse().unwrap(),
        ]);
        assert_eq!(name.unwrap_err(), NameErr::EmptyLabel);

        // domain with 256 characters long is invalid
        let a_50_character_long_name = "a-50-character-long--------------------------label"
            .parse::<Label>()
            .unwrap();
        assert_eq!(a_50_character_long_name.to_string().len(), 50);
        let len_256_name = Name::create(vec![
            a_50_character_long_name.clone(),
            a_50_character_long_name.clone(),
            a_50_character_long_name.clone(),
            a_50_character_long_name.clone(),
            a_50_character_long_name.clone(),
        ]); // sum_label_octets_len = 50 * 5 = 250; num_label_len = 5 + 1 = 6; 250 + 6 = 256 > 255
        assert_eq!(len_256_name.unwrap_err(), NameErr::LenLimit);
    }

    #[test]
    fn test_domain_from_str() {
        // "" is a valid domain name
        let domain: Name = "".parse().unwrap();
        assert_eq!(domain.to_string(), ".");

        // "." is a valid domain name
        let domain: Name = ".".parse().unwrap();
        assert_eq!(domain.to_string(), ".");

        // "example" is a valid domain name
        let domain: Name = "example".parse().unwrap();
        assert_eq!(domain.to_string(), "example.");

        // "example." is a valid domain name
        let domain: Name = "example.".parse().unwrap();
        assert_eq!(domain.to_string(), "example.");

        // "example.com" is a valid domain name
        let domain: Name = "example.com".parse().unwrap();
        assert_eq!(domain.to_string(), "example.com.");

        // "example.com." is a valid domain name
        let domain: Name = "example.com.".parse().unwrap();
        assert_eq!(domain.to_string(), "example.com.");

        // domains with more than 63 characters in a label are invalid
        let domain: Result<Name, FromStrErr> =
            "a-64-character-long-label----------------------------is-too-long".parse();
        assert_eq!(
            domain.unwrap_err(),
            FromStrErr::LabelFromStrErr(LabelFromStrErr::LenLimit)
        );
    }

    #[test]
    fn test_is_root() {
        // "" should be a root domain name
        let name: Name = "".parse().unwrap();
        assert_eq!(name.to_string(), ".");
        assert!(name.is_root());

        // "." should be a root domain name
        let name: Name = ".".parse().unwrap();
        assert_eq!(name.to_string(), ".");
        assert!(name.is_root());

        // DomainName::root() should return a DomainName with a single root label
        let name = Name::root();
        assert_eq!(name.to_string(), ".");
        assert!(name.is_root());
    }

    #[test]
    fn test_is_subdomain() {
        // "." is not subdomain of itself
        let subdomain = ".".parse::<Name>().unwrap();
        let domain = ".".parse::<Name>().unwrap();
        assert!(!subdomain.is_subdomain(&domain));

        // "example." is subdomain of "."
        let subdomain = "example.".parse::<Name>().unwrap();
        let domain = ".".parse::<Name>().unwrap();
        assert!(subdomain.is_subdomain(&domain));

        // "." is not subdomain of "example."
        let subdomain = ".".parse::<Name>().unwrap();
        let domain = "example.".parse::<Name>().unwrap();
        assert!(!subdomain.is_subdomain(&domain));

        // "example.com." is subdomain of "com."
        let subdomain = "example.com.".parse::<Name>().unwrap();
        let domain = "com.".parse::<Name>().unwrap();
        assert!(subdomain.is_subdomain(&domain));

        // "com." is not subdomain of "example.com."
        let subdomain = "com.".parse::<Name>().unwrap();
        let domain = "example.com.".parse::<Name>().unwrap();
        assert!(!subdomain.is_subdomain(&domain));

        // "example." is not subdomain of "com."
        let subdomain = "example.".parse::<Name>().unwrap();
        let domain = "com.".parse::<Name>().unwrap();
        assert!(!subdomain.is_subdomain(&domain));
    }
}
