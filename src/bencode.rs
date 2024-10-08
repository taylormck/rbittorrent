// NOTE: This file is currently unused, replaced by serde_bencode.
// Maybe I'll go back and implement it correctly later.

use serde_json;
use std::iter::Peekable;

pub fn decode<'a>(encoded_value: impl IntoIterator<Item = &'a u8>) -> serde_json::Value {
    let mut encoded_chars = encoded_value.into_iter().map(|b| *b as char).peekable();
    decode_next_bencoded_value(&mut encoded_chars)
}

fn decode_next_bencoded_value(
    encoded_bytes: &mut Peekable<impl Iterator<Item = char>>,
) -> serde_json::Value {
    if let Some(first_byte) = encoded_bytes.next() {
        return match first_byte {
            // NOTE: strings have the format "{length}:{content}", for example: "5:hello"
            d if d.is_ascii_digit() => {
                let mut length_digits = vec![d];

                for c in encoded_bytes.by_ref() {
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
                    let c = encoded_bytes
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
                if let Some(c) = encoded_bytes.peek() {
                    if *c == '-' {
                        digits.push('-');
                        encoded_bytes.next();
                    }
                }

                for c in encoded_bytes {
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

                while let Some(c) = encoded_bytes.peek() {
                    if *c == 'e' {
                        // Consume the e
                        encoded_bytes.next();
                        break;
                    }

                    result.push(decode_next_bencoded_value(encoded_bytes));
                }
                serde_json::Value::Array(result)
            }
            'd' => {
                let mut result = serde_json::Map::<String, serde_json::Value>::new();

                while let Some(c) = encoded_bytes.peek() {
                    if *c == 'e' {
                        // Consume the e
                        encoded_bytes.next();
                        break;
                    }

                    let key = match decode_next_bencoded_value(encoded_bytes) {
                        serde_json::Value::String(key) => key,
                        key => panic!("Non-string value used as key in dictionary: {}", key),
                    };

                    let value = decode_next_bencoded_value(encoded_bytes);

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

pub fn encode(val: serde_json::Value) -> Vec<u8> {
    match val {
        serde_json::Value::String(s) => format!("{}:{}", s.len(), s).into(),
        serde_json::Value::Number(n) => format!("i{}e", n).into(),
        serde_json::Value::Array(a) => format!(
            "l{}e",
            a.into_iter()
                .flat_map(encode)
                .map(|b| b as char)
                .collect::<String>()
        )
        .into(),
        serde_json::Value::Object(o) => format!(
            "d{}e",
            o.into_iter()
                .flat_map(
                    |(key, val)| vec![encode(serde_json::Value::String(key)), encode(val)]
                        .into_iter()
                        .flatten()
                )
                .map(|b| b as char)
                .collect::<String>()
        )
        .into(),
        _ => panic!("Tried to encode unsupported value type."),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_decodeing() {
        assert_eq!(decode("5:hello".as_bytes()), "hello");
        assert_eq!(decode("3:foo".as_bytes()), "foo");
    }

    #[test]
    fn test_decode_integer() {
        assert_eq!(decode("i5e".as_bytes()), 5);
        assert_eq!(decode("i42e".as_bytes()), 42);
        assert_eq!(decode("i-1e".as_bytes()), -1);
        assert_eq!(decode("i-1234e".as_bytes()), -1234);
    }

    #[test]
    fn test_decode_list() {
        assert_eq!(
            decode("l5:helloi52ee".as_bytes()),
            serde_json::Value::Array(vec![
                serde_json::Value::String("hello".to_string()),
                serde_json::Value::Number(52.into()),
            ]),
        );

        assert_eq!(
            decode("l3:fooe".as_bytes()),
            serde_json::Value::Array(vec![serde_json::Value::String("foo".to_string()),]),
        );

        assert_eq!(decode("le".as_bytes()), serde_json::Value::Array(vec![]),);
    }

    #[test]
    fn test_decode_dictionary() {
        let mut expected_map = serde_json::Map::new();
        expected_map.insert("foo".to_string(), "bar".into());
        expected_map.insert("hello".to_string(), 52.into());

        assert_eq!(
            decode("d3:foo3:bar5:helloi52ee".as_bytes()),
            serde_json::Value::Object(expected_map)
        );

        let expected_map = serde_json::Map::new();

        assert_eq!(
            decode("de".as_bytes()),
            serde_json::Value::Object(expected_map)
        );
    }

    #[test]
    fn test_encode_string() {
        assert_eq!(
            encode(serde_json::Value::String("hello".to_string())),
            "5:hello".as_bytes()
        );

        assert_eq!(
            encode(serde_json::Value::String("foo".to_string())),
            "3:foo".as_bytes()
        );

        assert_eq!(
            encode(serde_json::Value::String("".to_string())),
            "0:".as_bytes()
        );
        assert_eq!(
            encode(serde_json::Value::String("\x00".to_string())),
            vec![b'1', b':', b'\x00'],
        );
    }

    #[test]
    fn test_encode_integer() {
        assert_eq!(
            encode(serde_json::Value::Number(420.into())),
            "i420e".as_bytes()
        );
        assert_eq!(
            encode(serde_json::Value::Number((-69).into())),
            "i-69e".as_bytes()
        );
    }

    #[test]
    fn test_encode_list() {
        assert_eq!("le".as_bytes(), encode(serde_json::Value::Array(vec![])));

        let input = vec![
            serde_json::Value::Number(420.into()),
            serde_json::Value::String("hello".to_string()),
        ];

        let expected = "li420e5:helloe".as_bytes();

        assert_eq!(expected, encode(serde_json::Value::Array(input)));
    }

    #[test]
    fn test_encode_dictionary() {
        let mut input = serde_json::Map::new();

        let expected = "de".as_bytes();
        let actual = encode(serde_json::Value::Object(input.clone()));
        assert_eq!(expected, actual);

        input.insert("foo".into(), "bar".into());
        input.insert("baz".into(), 69.into());

        let expected = "d3:bazi69e3:foo3:bare".as_bytes();
        let actual = encode(serde_json::Value::Object(input.clone()));
        assert_eq!(expected, actual);

        input.insert("a".into(), "first".into());
        input.insert("z".into(), "last".into());

        let expected = "d1:a5:first3:bazi69e3:foo3:bar1:z4:laste".as_bytes();
        let actual = encode(serde_json::Value::Object(input));
        assert_eq!(expected, actual);
    }
}
