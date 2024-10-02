use std::{iter::Peekable, str::Chars};

pub fn decode(encoded_value: &str) -> serde_json::Value {
    let mut encoded_chars = encoded_value.chars().peekable();

    decode_next_bencoded_value(&mut encoded_chars)
}

fn decode_next_bencoded_value(encoded_chars: &mut Peekable<Chars>) -> serde_json::Value {
    if let Some(first_char) = encoded_chars.next() {
        return match first_char {
            // NOTE: strings have the format "{length}:{content}", for example: "5:hello"
            d if d.is_ascii_digit() => {
                let mut length_digits = vec![d];
                for c in encoded_chars.by_ref() {
                    match c {
                        c if c.is_ascii_digit() => length_digits.push(c),
                        ':' => break,
                        c => panic!("Unexpected character in string length: {}", c),
                    }
                }

                let string_length = length_digits
                    .into_iter()
                    .collect::<String>()
                    .parse::<i64>()
                    .unwrap();

                let mut string_chars = vec![];

                for _ in 0..string_length {
                    let c = encoded_chars
                        .next()
                        .expect("Unexpected end of content while reading string");
                    string_chars.push(c);
                }

                serde_json::Value::String(string_chars.into_iter().collect::<String>())
            }
            // NOTE: integers always start with 'i' and end with 'e'.
            // Negative integers have a '-' immediately after the 'i'.
            'i' => {
                let mut digits = vec![];

                // We only want to allow the `-` at the beginning of the integer.
                if let Some(c) = encoded_chars.peek() {
                    if *c == '-' {
                        digits.push('-');
                        encoded_chars.next();
                    }
                }

                for c in encoded_chars {
                    match c {
                        c if c.is_ascii_digit() => digits.push(c),
                        'e' => break,
                        _ => panic!("Unexpected character in integer"),
                    }
                }

                digits.into_iter().collect::<String>().parse().unwrap()
            }
            'l' => {
                let mut result = vec![];

                while let Some(c) = encoded_chars.peek() {
                    if *c == 'e' {
                        // Consume the e
                        encoded_chars.next();
                        break;
                    }

                    result.push(decode_next_bencoded_value(encoded_chars));
                }
                serde_json::Value::Array(result)
            }
            'd' => {
                let mut result = serde_json::Map::<String, serde_json::Value>::new();

                while let Some(c) = encoded_chars.peek() {
                    if *c == 'e' {
                        // Consume the e
                        encoded_chars.next();
                        break;
                    }

                    let key = match decode_next_bencoded_value(encoded_chars) {
                        serde_json::Value::String(key) => key,
                        key => panic!("Non-string value used as key in dictionary: {}", key),
                    };

                    let value = decode_next_bencoded_value(encoded_chars);

                    result.insert(key.to_string(), value);
                }

                serde_json::Value::Object(result)
            }
            c => {
                panic!("Unhandled encoded value: {}", c);
            }
        };
    }

    panic!("No value to decode");
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_decode_string() {
        assert_eq!(decode("5:hello"), "hello");
        assert_eq!(decode("3:foo"), "foo");
    }

    #[test]
    fn test_decode_integer() {
        assert_eq!(decode("i5e"), 5);
        assert_eq!(decode("i42e"), 42);
        assert_eq!(decode("i-1e"), -1);
        assert_eq!(decode("i-1234e"), -1234);
    }

    #[test]
    fn test_decode_list() {
        assert_eq!(
            decode("l5:helloi52ee"),
            serde_json::Value::Array(vec![
                serde_json::Value::String("hello".to_string()),
                serde_json::Value::Number(52.into()),
            ]),
        );

        assert_eq!(
            decode("l3:fooe"),
            serde_json::Value::Array(vec![serde_json::Value::String("foo".to_string()),]),
        );

        assert_eq!(decode("le"), serde_json::Value::Array(vec![]),);
    }

    #[test]
    fn test_decode_dictionary() {
        let mut expected_map = serde_json::Map::new();
        expected_map.insert("foo".to_string(), "bar".into());
        expected_map.insert("hello".to_string(), 52.into());

        assert_eq!(
            decode("d3:foo3:bar5:helloi52ee"),
            serde_json::Value::Object(expected_map)
        );

        let expected_map = serde_json::Map::new();

        assert_eq!(decode("de"), serde_json::Value::Object(expected_map));
    }
}
