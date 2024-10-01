use serde_json;
use std::env;

// Available if you need it!
// use serde_bencode

#[allow(dead_code)]
fn decode_bencoded_value(encoded_value: &str) -> serde_json::Value {
    let mut encoded_chars = encoded_value.chars().peekable();

    if let Some(first_char) = encoded_chars.peek() {
        return match first_char {
            // If encoded_value starts with a digit, it's a number
            d if d.is_digit(10) => {
                // Example: "5:hello" -> "hello"
                let colon_index = encoded_value.find(':').unwrap();
                let number_string = &encoded_value[..colon_index];
                let number = number_string.parse::<i64>().unwrap();
                let string = &encoded_value[colon_index + 1..colon_index + 1 + number as usize];

                serde_json::Value::String(string.to_string())
            }
            'i' => {
                // Skip the 'i'
                encoded_chars.next();

                let mut digits = encoded_chars.collect::<String>();
                if !digits.ends_with("e") {
                    panic!("Integer value did not end with 'e'");
                }

                digits.pop();

                digits.parse().expect("Not a valid number")
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
