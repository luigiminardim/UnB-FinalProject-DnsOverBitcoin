use std::{
    iter::Peekable,
    str::{Chars, FromStr},
};

use super::Record;

pub fn parse_str(input: &str) -> Vec<Record> {
    let input = &remove_comments(input);
    let input = &remove_parentesis(input);
    let input = remove_white_space(input);
    dbg!(&input);
    input
        .lines()
        .filter_map(|line| {
            Record::from_str(line)
                .inspect_err(|e| {
                    dbg!(e, line);
                })
                .ok()
        })
        .collect::<Vec<Record>>()
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
}
