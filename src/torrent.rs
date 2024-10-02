use crate::bencode;
use serde_json;
use std::fs;

#[derive(Clone, Debug)]
pub struct Torrent {
    pub announce: String,
    pub length: u64,
}

impl Torrent {
    pub fn from_file(path: &str) -> Self {
        let contents = fs::read(path).expect("Failed to read file.");
        let contents = contents.into_iter().map(|c| c as char).collect::<String>();
        let dict = bencode::decode(&contents);

        let data = match dict {
            serde_json::Value::Object(dict) => dict,
            _ => panic!("Invalid torrent file does not represent a bencoded dictionary."),
        };

        let announce = match data.get("announce") {
            Some(serde_json::Value::String(url)) => url.to_owned(),
            _ => panic!("Torrent file does not contain announce entry."),
        };

        let info = match data.get("info") {
            Some(serde_json::Value::Object(info)) => info,
            _ => panic!("Torrent file does not contain info entry."),
        };

        let length = match info.get("length") {
            Some(serde_json::Value::Number(len)) => len
                .as_u64()
                .expect("Torrent file contains invalid length entry."),
            _ => panic!("Torrent file does not contain length entry."),
        };

        Self { announce, length }
    }
}
