use bittorrent_starter_rust::{bencode, peers::fetch_peers, Torrent};
use std::env;

// Usage: your_bittorrent.sh decode "<encoded_value>"
fn main() {
    let args: Vec<String> = env::args().collect();
    let command = &args[1];

    match command.as_str() {
        "decode" => {
            let encoded_value = &args[2];
            let decoded_value = bencode::decode(encoded_value.as_bytes());
            println!("{}", decoded_value);
        }
        "info" => {
            let file_path = &args[2];
            let torrent = Torrent::from_file(file_path).unwrap();
            println!("Tracker URL: {}", torrent.announce);
            println!("Length: {}", torrent.length);
            println!("Info Hash: {}", torrent.hash);
            println!("Piece Length: {}", torrent.piece_length);
            println!("Pieces: \n{}", torrent.piece_hashes.join("\n"));
        }
        "peers" => {
            let file_path = &args[2];
            let torrent = Torrent::from_file(file_path).unwrap();
            let peers = fetch_peers(&torrent).unwrap();

            peers.iter().for_each(|peer| println!("{}", peer));
        }
        _ => {
            println!("unknown command: {}", args[1])
        }
    }
}
