use std::str::FromStr;

use super::Record;

fn parse_str(input: &str) -> Vec<Record> {
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

fn remove_white_space(input: &str) -> String {
    let mut result = String::with_capacity(input.len());
    let mut iter = input.chars().peekable();
    while let Some(ch) = iter.next() {
        let next = iter.peek();
        if ch == '\n' {
            result.push('\n');
        } else if ch.is_whitespace() {
            let next_is_white_space = next.map_or(true, |next| next.is_whitespace());
            if next_is_white_space {
            } else {
                result.push(' ');
            }
        } else {
            result.push(ch);
        }
    }
    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a() {
        assert!('\t'.is_whitespace());
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
    fn test_extra_spaces() {
        // extra spaces
        let input = "example.com.  0 IN\tA 127.0.0.1 \r\nexample.com. 0 IN CNAME cname.com.";
        let expected: Vec<Record> = vec![
            "example.com. 0 IN A 127.0.0.1".parse().unwrap(),
            "example.com. 0 IN CNAME cname.com.".parse().unwrap(),
        ];
        let output = parse_str(input);
        assert_eq!(expected, output);
    }
}
