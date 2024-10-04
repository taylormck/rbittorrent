use serde_bencode::value::Value as BValue;
use sha1::{Digest, Sha1};
use std::fs;

#[derive(Clone, Debug, PartialEq)]
pub struct Torrent {
    pub announce: String,
    pub length: i64,
    pub hash: String,
    pub piece_length: i64,
    pub piece_hashes: Vec<String>,
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

        let piece_length = match info.get("piece length".as_bytes()) {
            Some(BValue::Int(len)) => *len,
            _ => panic!("Torrent file does not contain piece length entry."),
        };

        let pieces = match info.get("pieces".as_bytes()) {
            Some(BValue::Bytes(bytes)) => bytes,
            _ => panic!("Torrent file does not contain pieces entry."),
        };

        let piece_hashes = pieces.chunks(20).map(const_hex::encode).collect();

        let encoded_info = serde_bencode::to_bytes(&BValue::Dict(info.clone()))?;

        let hash = calculate_hash(&encoded_info);

        Ok(Self {
            announce,
            length,
            hash,
            piece_length,
            piece_hashes,
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
        let input_info = "d6:lengthi420e12:piece lengthi512e6:pieces20:01234567890123456789e";
        let expected_hash = calculate_hash(input_info.as_bytes());
        let input = format!("d8:announce8:fake_url4:info{}e", input_info);

        let expected_torrent = Torrent {
            announce: "fake_url".to_string(),
            length: 420_i64,
            hash: expected_hash,
            piece_length: 512,
            piece_hashes: vec!["3031323334353637383930313233343536373839".to_string()],
        };

        let actual_torrent = Torrent::new(input.as_bytes()).unwrap();

        assert_eq!(expected_torrent, actual_torrent);
    }
}
