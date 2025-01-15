use std::{
    iter::Peekable,
    str::{Chars, FromStr},
};

use super::{Class, Record, Ttl};

// TODO: Use parser combinator or bnfc
pub fn parse_str(input: &str) -> Vec<Record> {
    let input = &remove_comments(input);
    let input = &remove_parentesis(input);
    let input = remove_white_space(input);
    let mut records = Vec::new();
    for line in input.lines() {
        let last_record = records.last();
        let record = parse_record(line, last_record);
        match record {
            Some(record) => {
                records.push(record);
            }
            None => {
                continue;
            }
        }
    }
    records
}

fn remove_comments(input: &str) -> String {
    let mut result = String::with_capacity(input.len());
    let mut iter = input.chars().peekable();
    let mut in_comment = false;
    loop {
        if let Some(string) = consume_quoted_string(&mut iter) {
            result.push_str(&string);
        } else if let Some(ch) = iter.next() {
            if ch == '\n' {
                in_comment = false;
                result.push(ch);
            } else if in_comment {
                continue;
            } else if ch == ';' {
                in_comment = true;
                continue;
            } else {
                result.push(ch);
            }
        } else {
            break;
        }
    }
    result
}

fn remove_parentesis(input: &str) -> String {
    let mut result = String::with_capacity(input.len());
    let mut iter = input.chars().peekable();
    let mut is_in_parenthesis = false;
    loop {
        if let Some(string) = consume_quoted_string(&mut iter) {
            result.push_str(&string);
        } else if let Some(ch) = iter.next() {
            if ch == '\n' {
                if is_in_parenthesis {
                    result.push(' ');
                } else {
                    result.push('\n');
                }
            } else if ch == '(' {
                is_in_parenthesis = true;
                continue;
            } else if ch == ')' {
                is_in_parenthesis = false;
                continue;
            } else {
                result.push(ch);
            }
        } else {
            break;
        }
    }
    result
}

fn remove_white_space(input: &str) -> String {
    let mut result = String::with_capacity(input.len());
    let mut iter = input.chars().peekable();
    loop {
        if let Some(string) = consume_quoted_string(&mut iter) {
            result.push_str(&string);
        } else if let Some(ch) = iter.next() {
            let next = iter.peek().cloned();
            if ch == '\n' {
                result.push('\n');
            } else if ch.is_whitespace() {
                let next_is_white_space = next.map_or(true, |next| next.is_whitespace());
                if !next_is_white_space {
                    result.push(' ');
                }
            } else {
                result.push(ch);
            }
        } else {
            break;
        }
    }
    result
}

fn consume_quoted_string(iter: &mut Peekable<Chars>) -> Option<String> {
    if iter.by_ref().peek() != Some(&'"') {
        return None;
    }
    iter.next();
    let mut result = "\"".to_string();
    while let Some(ch) = iter.next() {
        if ch == '\\' {
            result.push(ch);
            if let Some(ch) = iter.next() {
                result.push(ch);
            }
        } else {
            result.push(ch);
            if ch == '"' {
                break;
            }
        }
    }
    Some(result)
}

fn parse_record(line: &str, last_record: Option<&Record>) -> Option<Record> {
    let (domain_name_part, rr_part) = line.split_once(' ')?;
    let domain_name_part = if domain_name_part.is_empty() {
        last_record
            .as_ref()
            .map(|last_record| last_record.name().to_string())
    } else {
        Some(domain_name_part.to_string())
    }?;
    let (ttl, class, rest) = separate_rr_fields(rr_part, last_record)?;
    let input = format!("{} {} {} {}", domain_name_part, ttl, class, rest);
    Record::from_str(&input).ok()
}

/// Return (ttl, class, ...rest)
fn separate_rr_fields(
    rr_part: &str,
    last_record: Option<&Record>,
) -> Option<(String, String, String)> {
    let mut parts = rr_part.split_whitespace().peekable();
    if let Some(Ok(ttl)) = parts.peek().map(|s| Ttl::from_str(s)) {
        // If the first part is a number
        parts.next();
        if let Some(Ok(class)) = parts.peek().map(|s| Class::from_str(s)) {
            // If the second part is a class
            parts.next();
            let rest = parts.collect::<Vec<&str>>().join(" ");
            return Some((ttl.to_string(), class.to_string(), rest));
        } else if let Some(last_record) = last_record {
            // If the second part is not a class, but the first part is a number
            let class = last_record.class().to_string();
            let rest = parts.collect::<Vec<&str>>().join(" ");
            return Some((last_record.ttl().to_string(), class, rest));
        } else {
            return None;
        }
    } else if let Some(Ok(class)) = parts.peek().map(|s| Class::from_str(s)) {
        // If the first part is a class
        parts.next();
        if let Some(Ok(ttl)) = parts.peek().map(|s| Ttl::from_str(s)) {
            // If the second part is a number
            parts.next();
            let rest = parts.collect::<Vec<&str>>().join(" ");
            return Some((ttl.to_string(), class.to_string(), rest));
        } else if let Some(last_record) = last_record {
            // If the second part is not a number, but the first part is a class
            let ttl = last_record.ttl().to_string();
            let rest = parts.collect::<Vec<&str>>().join(" ");
            return Some((ttl, class.to_string(), rest));
        } else {
            return None;
        }
    } else if let Some(last_record) = last_record {
        // If the first part is not a class nor a number
        let ttl = last_record.ttl().to_string();
        let class = last_record.class().to_string();
        let rest = parts.collect::<Vec<&str>>().join(" ");
        return Some((ttl, class, rest));
    } else {
        return None;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a() {
        assert_eq!(
            consume_quoted_string(&mut "\"Hello World!\"".chars().peekable()),
            Some("\"Hello World!\"".to_string())
        );
    }

    #[test]
    fn test_parse_str() {
        // single line
        let input = "example.com. 0 IN A 127.0.0.1";
        let expected: Vec<Record> = vec!["example.com. 0 IN A 127.0.0.1".parse().unwrap()];
        let output = parse_str(input);
        assert_eq!(expected, output);

        // multi line
        let input = "example.com. 0 IN A 127.0.0.1\nexample.com. 0 IN CNAME cname.com.";
        let expected: Vec<Record> = vec![
            "example.com. 0 IN A 127.0.0.1".parse().unwrap(),
            "example.com. 0 IN CNAME cname.com.".parse().unwrap(),
        ];
        let output = parse_str(input);
        assert_eq!(expected, output);

        // multi line with invalid line
        let input = r#"
example.com. 0 IN A 127.0.0.1
INVALID
example.com. 0 IN CNAME cname.com.
"#;
        let expected: Vec<Record> = vec![
            "example.com. 0 IN A 127.0.0.1".parse().unwrap(),
            "example.com. 0 IN CNAME cname.com.".parse().unwrap(),
        ];
        let output = parse_str(input);
        assert_eq!(expected, output);
    }

    #[test]
    fn test_parse_extra_spaces() {
        let input = "example.com.  0 IN\tA 127.0.0.1 \r\nexample.com. 0 IN CNAME cname.com.";
        let expected: Vec<Record> = vec![
            "example.com. 0 IN A 127.0.0.1".parse().unwrap(),
            "example.com. 0 IN CNAME cname.com.".parse().unwrap(),
        ];
        let output = parse_str(input);
        assert_eq!(expected, output);
    }

    #[test]
    fn test_parse_extra_spaces_in_string() {
        let input = r#"
example.com. 0 IN TXT "Hello, World!"
"#;
        let expected: Vec<Record> =
            vec!["example.com. 0 IN TXT \"Hello, World!\"".parse().unwrap()];
        let output = parse_str(input);
        assert_eq!(expected, output);
    }

    #[test]
    fn test_remove_parentesis() {
        let input = r#"
example.com.    (   0
                    IN 
                    A 
                    127.0.0.1)
example.com. 0 (IN
                CNAME
                cname.com.
            )

example.com. 0 (
                IN
                TXT
                "Hello, World!")
"#;
        let expected: Vec<Record> = vec![
            "example.com. 0 IN A 127.0.0.1".parse().unwrap(),
            "example.com. 0 IN CNAME cname.com.".parse().unwrap(),
            "example.com. 0 IN TXT \"Hello, World!\"".parse().unwrap(),
        ];
        let output = parse_str(input);
        assert_eq!(expected, output);
    }

    #[test]
    fn test_remove_comments() {
        let input = r#"
example.com.    (   0; comment
                    IN ; comment
                    A ; comment
                    127.0.0.1); comment
example.com. 0 (IN ; comment
                CNAME ; comment
                cname.com.; comment
            )

; comment
example.com. 0 (
                IN ; comment
                TXT ; comment
                "Hello; World!") ; comment
"#;
        let expected: Vec<Record> = vec![
            "example.com. 0 IN A 127.0.0.1".parse().unwrap(),
            "example.com. 0 IN CNAME cname.com.".parse().unwrap(),
            "example.com. 0 IN TXT \"Hello; World!\"".parse().unwrap(),
        ];
        let output = parse_str(input);
        assert_eq!(expected, output);
    }

    #[test]
    fn test_omit_domain_name() {
        let input = r#"
example.com.    0 IN A 127.0.0.1
                0 IN CNAME cname.com.
                0 IN TXT "Hello, World!"
"#;
        let expected: Vec<Record> = vec![
            "example.com. 0 IN A 127.0.0.1".parse().unwrap(),
            "example.com. 0 IN CNAME cname.com.".parse().unwrap(),
            "example.com. 0 IN TXT \"Hello, World!\"".parse().unwrap(),
        ];
        let output = parse_str(input);
        assert_eq!(expected, output);
    }

    #[test]
    fn test_parse_record_with_rr_in_different_orders() {
        let input = r#"
; [<domain-name>]   [<TTL>]     [<class>]   <type> <RDATA>
example.com.        0           IN          A 0.1.1.1
example.com.        0                       A 0.1.1.0
example.com.                    IN          A 0.1.0.1
example.com.                                A 0.1.0.0
                    0           IN          A 0.0.1.1
                    0                       A 0.0.1.0
                                IN          A 0.0.0.1
                                            A 0.0.0.0  

; [<domain-name>]   [<class>]   [<TTL>]     <type> <RDATA>
example.com.        IN          0           A 0.1.1.1
example.com.        IN                      A 0.1.1.0
example.com.                    0           A 0.1.0.1
example.com.                                A 0.1.0.0
                    IN          0           A 0.0.1.1
                    IN                      A 0.0.1.0
                                0           A 0.0.0.1
                                            A 0.0.0.0 
"#;
        let expected: Vec<Record> = vec![
            "example.com. 0 IN A 0.1.1.1",
            "example.com. 0 IN A 0.1.1.0",
            "example.com. 0 IN A 0.1.0.1",
            "example.com. 0 IN A 0.1.0.0",
            "example.com. 0 IN A 0.0.1.1",
            "example.com. 0 IN A 0.0.1.0",
            "example.com. 0 IN A 0.0.0.1",
            "example.com. 0 IN A 0.0.0.0",
            "example.com. 0 IN A 0.1.1.1",
            "example.com. 0 IN A 0.1.1.0",
            "example.com. 0 IN A 0.1.0.1",
            "example.com. 0 IN A 0.1.0.0",
            "example.com. 0 IN A 0.0.1.1",
            "example.com. 0 IN A 0.0.1.0",
            "example.com. 0 IN A 0.0.0.1",
            "example.com. 0 IN A 0.0.0.0",
        ]
        .iter()
        .map(|s| Record::from_str(s).unwrap())
        .collect();
        let output = parse_str(input);
        expected
            .iter()
            .zip(output.iter())
            .for_each(|(expected, output)| {
                assert_eq!(
                    expected,
                    output,
                    "could't parse \"{}\"",
                    expected.to_string(),
                );
            });
    }
}
