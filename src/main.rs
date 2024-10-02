use bittorrent_starter_rust::{bencode, torrent};
use std::env;

// Usage: your_bittorrent.sh decode "<encoded_value>"
fn main() {
    let args: Vec<String> = env::args().collect();
    let command = &args[1];

    match command.as_str() {
        "decode" => {
            let encoded_value = &args[2];
            let decoded_value = bencode::decode(encoded_value);
            println!("{}", decoded_value);
        }
        "info" => {
            let file_path = &args[2];
            let torrent = torrent::Torrent::from_file(file_path);
            println!("Tracker URL: {}", torrent.announce);
            println!("Length: {}", torrent.length);
        }
        _ => {
            println!("unknown command: {}", args[1])
        }
    }
}
