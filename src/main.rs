use bittorrent_starter_rust::{bencode, torrent};
use serde_bencode::value::Value as BValue;
use std::{env, fmt::Display, io::Read, process};

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
            let torrent = torrent::Torrent::from_file(file_path).unwrap();
            println!("Tracker URL: {}", torrent.announce);
            println!("Length: {}", torrent.length);
            println!("Info Hash: {}", torrent.hash);
            println!("Piece Length: {}", torrent.piece_length);
            println!("Pieces: \n{}", torrent.piece_hashes.join("\n"));
        }
        "peers" => {
            let file_path = &args[2];
            let torrent = torrent::Torrent::from_file(file_path).unwrap();

            // This may look scary, but all it does is stick a '%' in between
            // every pair of characters.
            let info_hash = torrent
                .hash
                .chars()
                .enumerate()
                .flat_map(|(i, c)| {
                    if i % 2 == 0 { Some('%') } else { None }
                        .into_iter()
                        .chain(std::iter::once(c))
                })
                .collect::<String>();

            let peer_id = "00112233445566778899";
            let port = 6881;
            let uploaded = 0;
            let downloaded = 0;
            let compact = 1;

            let url = format!(
                "{}?info_hash={}&peer_id={}&port={}&uploaded={}&downloaded={}&left={}&compact={}",
                torrent.announce,
                info_hash,
                peer_id,
                port,
                uploaded,
                downloaded,
                torrent.length,
                compact
            );

            let mut body = Vec::<u8>::new();
            match reqwest::blocking::get(url) {
                Ok(mut res) => match res.read_to_end(&mut body) {
                    Ok(_) => {}
                    Err(err) => eprintln!("Error reading response: {}", err),
                },
                Err(err) => {
                    eprintln!("Error fetching announce URL: {}", err);
                    process::exit(1);
                }
            }

            let body = match serde_bencode::from_bytes::<BValue>(&body) {
                Ok(BValue::Dict(body)) => body,
                _ => {
                    eprintln!("Response body was not a dictionary");
                    process::exit(1);
                }
            };

            let peers = match body.get("peers".as_bytes()) {
                Some(BValue::Bytes(peers)) => peers,
                _ => {
                    eprintln!("No peers in response");
                    process::exit(1);
                }
            };

            let peers: Vec<IpAddress> = peers
                .chunks(6)
                .map(|chunk| {
                    let mut peer = [0; 6];
                    peer.clone_from_slice(chunk);
                    IpAddress::from_bytes(&peer)
                })
                .collect();

            peers.iter().for_each(|peer| println!("{}", peer));
        }
        _ => {
            println!("unknown command: {}", args[1])
        }
    }
}

#[derive(Debug, PartialEq, Eq, Clone)]
struct IpAddress {
    pub address: [u8; 4],
    pub port: u16,
}

impl IpAddress {
    pub fn from_bytes(bytes: &[u8; 6]) -> Self {
        Self {
            address: bytes[0..4]
                .iter()
                .map(|b| u8::from_be_bytes([*b]))
                .collect::<Vec<u8>>()
                .try_into()
                .unwrap(),
            port: u16::from_be_bytes(bytes[4..6].try_into().unwrap()),
        }
    }
}

impl Display for IpAddress {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}.{}.{}.{}:{}",
            self.address[0], self.address[1], self.address[2], self.address[3], self.port
        )
    }
}
