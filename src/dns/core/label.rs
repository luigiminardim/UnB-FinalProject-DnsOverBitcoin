use std::str::FromStr;

/// A label is zero to 63 octets in length. The domain name of a node is the
/// list of the labels
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Label(String);

const MAX_LABEL_LEN: usize = 63;

impl Label {
    /// One label is reserved, and that is the null (i.e., zero length) label
    /// used for the root.
    pub fn null() -> Self {
        Label("".to_string())
    }

    /// Check if it's the null (i.e., zero length) label used for the root.
    pub fn is_null(&self) -> bool {
        self.0.is_empty()
    }

    /// Get the length of the label. The "example" label has a length of 7.
    pub fn len(&self) -> usize {
        self.0.len()
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum LabelFromStrErr {
    LenLimit,
}
impl FromStr for Label {
    type Err = LabelFromStrErr;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if s.len() > MAX_LABEL_LEN {
            Err(Self::Err::LenLimit)
        } else {
            Ok(Label(s.to_string().to_lowercase()))
        }
    }
}

impl ToString for Label {
    fn to_string(&self) -> String {
        self.0.clone()
    }
}

#[cfg(test)]
mod label_tests {
    use super::*;

    #[test]
    fn test_label_from_str() {
        // The null label is a valid label
        let label: Label = "".parse().unwrap();
        assert_eq!(label.to_string(), "");

        // "example" is a valid label
        let label: Label = "example".parse().unwrap();
        assert_eq!(label.to_string(), "example");

        // a label with 64 characters is invalid
        let label: Result<Label, LabelFromStrErr> =
            "a-64-character-long-label----------------------------is-too-long".parse();
        assert!(label.is_err());
    }

    #[test]
    fn test_is_null() {
        let label: Label = "".parse().unwrap();
        assert_eq!(label.is_null(), true);

        let label: Label = "example".parse().unwrap();
        assert_eq!(label.is_null(), false);

        let label: Label = Label::null();
        assert_eq!(label.is_null(), true);
    }

    #[test]
    fn test_eq() {
        let label1: Label = "".parse().unwrap();
        let label2: Label = "".parse().unwrap();
        assert_eq!(label1, label2);

        let label1: Label = "example".parse().unwrap();
        let label2: Label = "example".parse().unwrap();
        assert_eq!(label1, label2);

        let label1: Label = "EXAMPLE".parse().unwrap();
        let label2: Label = "example".parse().unwrap();
        assert_eq!(label1, label2);

        let label1: Label = "".parse().unwrap();
        let label2: Label = "example".parse().unwrap();
        assert_ne!(label1, label2);
    }
}
