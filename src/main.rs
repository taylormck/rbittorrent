use bittorrent_starter_rust::{bencode, peers, Torrent};
use clap::{Parser, Subcommand};
use std::net::TcpStream;

#[derive(Clone, Debug, Parser)]
#[command(version, about, long_about = None)]
#[command(propagate_version = true)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Clone, Debug, Subcommand)]
enum Commands {
    Decode {
        encoded_value: String,
    },
    Info {
        file_path: String,
    },
    Peers {
        file_path: String,
    },
    Handshake {
        file_path: String,
        peer_ip: String,
    },
    #[command(name = "download_piece")]
    DownloadPiece {
        #[arg(short, long = "out")]
        output_path: Option<String>,
        file_path: String,
        piece_index: usize,
    },
}

// Usage: your_bittorrent.sh decode "<encoded_value>"
fn main() {
    let cli = Cli::parse();

    match &cli.command {
        Commands::Decode { encoded_value } => {
            let decoded_value = bencode::decode(encoded_value.as_bytes());
            println!("{}", decoded_value);
        }
        Commands::Info { file_path } => {
            let torrent = Torrent::from_file(file_path).unwrap();
            println!("Tracker URL: {}", torrent.announce);
            println!("Length: {}", torrent.length);
            println!("Info Hash: {}", torrent.hash);
            println!("Piece Length: {}", torrent.piece_length);
            println!("Pieces: \n{}", torrent.piece_hashes.join("\n"));
        }
        Commands::Peers { file_path } => {
            let torrent = Torrent::from_file(file_path).unwrap();
            let torrent_peers = peers::fetch_peers(&torrent).unwrap();

            torrent_peers.iter().for_each(|peer| println!("{}", peer));
        }
        Commands::Handshake { file_path, peer_ip } => {
            let torrent = Torrent::from_file(file_path).unwrap();

            let mut stream = TcpStream::connect(peer_ip).unwrap();
            let result = peers::shake_hands(&mut stream, &torrent).unwrap();

            println!("Peer ID: {}", result);
        }
        Commands::DownloadPiece {
            output_path: _output_path,
            file_path,
            piece_index: _piece_index,
        } => {
            let torrent = Torrent::from_file(file_path).unwrap();
            let torrent_peers = peers::fetch_peers(&torrent).unwrap();
            let peer_ip = torrent_peers[0];

            let mut stream = TcpStream::connect(peer_ip).unwrap();
            peers::shake_hands(&mut stream, &torrent).unwrap();

            todo!();
        }
    }
}
