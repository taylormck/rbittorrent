use bittorrent_starter_rust::{
    bencode,
    peers::{self, PeerMessage},
    Torrent,
};
use clap::{Parser, Subcommand};
use tokio::{
    // io::{AsyncRead, AsyncWrite},
    net::TcpStream,
};

#[derive(Clone, Debug, Parser)]
#[command(version, about, long_about = None)]
#[command(propagate_version = true)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Clone, Debug, Subcommand)]
#[clap(rename_all = "snake_case")]
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
    DownloadPiece {
        #[arg(short, long = "out")]
        output_path: Option<String>,
        file_path: String,
        piece_index: usize,
    },
}

// Usage: your_bittorrent.sh decode "<encoded_value>"
#[tokio::main]
async fn main() {
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

            let mut stream = match TcpStream::connect(peer_ip).await {
                Ok(stream) => stream,
                Err(err) => {
                    eprintln!("Error connecting to peer: {}", err);
                    std::process::exit(1);
                }
            };

            match peers::shake_hands(&mut stream, &torrent).await {
                Ok(result) => println!("Peer ID: {}", result),
                Err(err) => {
                    eprintln!("Error shaking hands: {}", err);
                    std::process::exit(1);
                }
            }
        }
        Commands::DownloadPiece {
            output_path: _output_path,
            file_path,
            piece_index: _piece_index,
        } => {
            let torrent = Torrent::from_file(file_path).unwrap();
            let torrent_peers = peers::fetch_peers(&torrent).unwrap();
            let peer_ip = torrent_peers[0];

            let mut stream = match TcpStream::connect(peer_ip).await {
                Ok(stream) => stream,
                Err(err) => {
                    eprintln!("Error connecting to peer: {}", err);
                    std::process::exit(1);
                }
            };

            println!("Handshake completed with peer: {}", peer_ip);

            while let Ok(message) = PeerMessage::read(&mut stream).await {
                println!("Message received: {:?}", message);

                if let Err(err) = message.process(&mut stream).await {
                    eprintln!("Error processing message: {}", err);
                    std::process::exit(1);
                }
            }
        }
    }
}
