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
    InvalidChar,
}
impl FromStr for Label {
    type Err = LabelFromStrErr;

    /// <label> ::= <letter> [ [ <ldh-str> ] <let-dig> ]
    /// <ldh-str> ::= <let-dig-hyp> | <let-dig-hyp> <ldh-str>
    /// <let-dig-hyp> ::= <let-dig> | "-"
    /// <let-dig> ::= <letter> | <digit>
    /// <letter> ::= any one of the 52 alphabetic characters A through Z in upper case and a through z in lower case
    /// <digit> ::= any one of the ten digits 0 through 9
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        fn is_letter(c: char) -> bool {
            c.is_ascii_alphabetic()
        }
        fn is_digit(c: char) -> bool {
            c.is_ascii_digit()
        }
        fn is_hyphen(c: char) -> bool {
            c == '-'
        }
        fn is_valid_char(c: char) -> bool {
            is_letter(c) || is_digit(c) || is_hyphen(c)
        }
        fn starts_with_letter(c: &str) -> bool {
            c.chars().next().map(is_letter).unwrap_or(false)
        }
        fn ends_with_letter_or_digit(c: &str) -> bool {
            c.chars()
                .last()
                .map(|c| is_letter(c) || is_digit(c))
                .unwrap_or(false)
        }
        fn has_valid_chars(c: &str) -> bool {
            c.chars().all(is_valid_char)
        }
        if s.len() > MAX_LABEL_LEN {
            Err(Self::Err::LenLimit)
        } else if s.len() == 0 {
            return Ok(Label::null());
        } else if starts_with_letter(s) && ends_with_letter_or_digit(s) && has_valid_chars(s) {
            Ok(Label(s.to_string().to_lowercase()))
        } else {
            Err(Self::Err::InvalidChar)
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

        // whitespaces in label is invalid
        let label: Result<Label, LabelFromStrErr> = "example ".parse();
        assert!(label.is_err());

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
