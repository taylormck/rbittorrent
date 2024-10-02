use serde_bencode::value::Value as BValue;
use sha1::{Digest, Sha1};
use std::fs;

#[derive(Clone, Debug, PartialEq)]
pub struct Torrent {
    pub announce: String,
    pub length: i64,
    pub hash: String,
}

impl Torrent {
    pub fn from_file(path: &str) -> Result<Self, serde_bencode::Error> {
        let contents = fs::read(path).expect("Failed to read file.");

        Self::new(&contents)
    }

    pub fn new(contents: &[u8]) -> Result<Self, serde_bencode::Error> {
        let dict = serde_bencode::from_bytes::<BValue>(contents);

        let data = match dict {
            Ok(BValue::Dict(dict)) => dict,
            _ => panic!("Invalid torrent file does not represent a bencoded dictionary."),
        };

        let announce = match data.get("announce".as_bytes()) {
            Some(BValue::Bytes(url)) => String::from_utf8_lossy(url).to_string(),
            _ => panic!("Torrent file does not contain announce entry."),
        };

        let info = match data.get("info".as_bytes()) {
            Some(BValue::Dict(info)) => info,
            _ => panic!("Torrent file does not contain info entry."),
        };

        let length = match info.get("length".as_bytes()) {
            Some(BValue::Int(len)) => *len,
            _ => panic!("Torrent file does not contain length entry."),
        };

        let encoded_info = serde_bencode::to_bytes(&BValue::Dict(info.clone()))?;

        let hash = calculate_hash(&encoded_info);

        Ok(Self {
            announce,
            length,
            hash,
        })
    }
}

fn calculate_hash(input: &[u8]) -> String {
    let mut hasher = Sha1::new();
    hasher.update(input);
    format!("{:x}", hasher.finalize())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_torrent_from_bytes() {
        let input_info = "d6:lengthi420ee";
        let expected_hash = calculate_hash(input_info.as_bytes());
        let input = format!("d8:announce8:fake_url4:info{}e", input_info);

        let expected_torrent = Torrent {
            announce: "fake_url".to_string(),
            length: 420_i64,
            hash: expected_hash,
        };

        let actual_torrent = Torrent::new(input.as_bytes()).unwrap();

        assert_eq!(expected_torrent, actual_torrent);
    }
}
