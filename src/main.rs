// use hex::encode;
// use serde_json;
use std::env;

// Available if you need it!
// use serde_bencode

#[allow(dead_code)]
fn decode_bencoded_value(encoded_value: &str) -> serde_json::Value {
    let mut encoded_chars = encoded_value.chars().peekable();

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

                // Example: "5:hello" -> "hello"
                // let colon_index = encoded_value.find(':').unwrap();
                // let number_string = &encoded_value[..colon_index];
                // let number = number_string.parse::<i64>().unwrap();
                // let string = &encoded_value[colon_index + 1..colon_index + 1 + number as usize];
                //
                // serde_json::Value::String(string.to_string())
            }
            'i' => {
                let mut digits = vec![];

                for c in encoded_chars {
                    match c {
                        c if c.is_ascii_digit() => digits.push(c),
                        'e' => break,
                        _ => panic!("Unexpected character in integer"),
                    }
                }

                digits.into_iter().collect::<String>().parse().unwrap()
            }
            _ => {
                panic!("Unhandled encoded value: {}", encoded_value);
            }
        };
    }

    panic!("No value to decode");
}

// Usage: your_bittorrent.sh decode "<encoded_value>"
fn main() {
    let args: Vec<String> = env::args().collect();
    let command = &args[1];

    if command == "decode" {
        let encoded_value = &args[2];
        let decoded_value = decode_bencoded_value(encoded_value);
        println!("{}", decoded_value);
    } else {
        println!("unknown command: {}", args[1])
    }
}
