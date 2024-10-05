use bittorrent_starter_rust::{bencode, peers, Torrent};
use std::{env, net::SocketAddrV4, str::FromStr};

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
            let torrent_peers = peers::fetch_peers(&torrent).unwrap();

            torrent_peers.iter().for_each(|peer| println!("{}", peer));
        }
        "handshake" => {
            let file_path = &args[2];
            let torrent = Torrent::from_file(file_path).unwrap();

            let peer_ip = SocketAddrV4::from_str(&args[3]).unwrap();
            let result = peers::shake_hands(peer_ip, &torrent).unwrap();

            println!("Peer ID: {}", result);
        }
        _ => {
            println!("unknown command: {}", args[1])
        }
    }
}
