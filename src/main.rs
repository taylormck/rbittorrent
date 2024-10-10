use bittorrent_starter_rust::{
    bencode,
    peers::{self, PeerMessage},
    FileInfo, Torrent,
};
use clap::{Parser, Subcommand};
use tokio::net::TcpStream;

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
            output_path,
            file_path,
            piece_index,
        } => {
            let torrent = Torrent::from_file(file_path).unwrap();

            if *piece_index >= torrent.piece_hashes.len() {
                eprintln!("Invalid piece index");
                std::process::exit(1);
            }

            // Forcibly remove all the pieces except the one we want to download.
            let piece_hash = torrent.piece_hashes[*piece_index].clone();
            let last_piece_index = torrent.piece_hashes.len() - 1;

            let length = match piece_index {
                i if *i == last_piece_index => torrent.length % torrent.piece_length,
                _ => torrent.piece_length,
            };

            let torrent = Torrent {
                announce: torrent.announce.clone(),
                length,
                hash: torrent.hash,
                piece_length: torrent.piece_length,
                piece_hashes: vec![piece_hash],
            };

            let torrent_peers = peers::fetch_peers(&torrent).unwrap();
            let peer_ip = torrent_peers[0];

            let output_path = output_path.clone().unwrap_or("/tmp/output".to_string());
            let mut file_info = FileInfo::new(output_path, &torrent);

            let mut stream = match TcpStream::connect(peer_ip).await {
                Ok(stream) => stream,
                Err(err) => {
                    eprintln!("Error connecting to peer: {}", err);
                    std::process::exit(1);
                }
            };

            eprintln!("Shaking hands...");
            match peers::shake_hands(&mut stream, &torrent).await {
                Ok(result) => eprintln!("Peer ID: {}", result),
                Err(err) => {
                    eprintln!("Error shaking hands: {}", err);
                    std::process::exit(1);
                }
            }

            eprintln!("Starting download...");
            loop {
                let message = PeerMessage::read(&mut stream).await;

                if let Err(err) = message {
                    eprintln!("Error reading message: {}", err);
                    std::process::exit(1);
                }

                let message = message.unwrap();

                eprintln!("Message received: {:?}", message.id);

                if let Err(err) = message.process(&mut stream, &mut file_info).await {
                    eprintln!("Error processing message: {}", err);
                    std::process::exit(1);
                }

                // TODO: probably a more performant way to handle this
                if file_info.is_complete() {
                    break;
                }
            }

            if let Err(err) = file_info.save_to_disk().await {
                eprintln!("Unable to save file to disk: {}", err);
                std::process::exit(1);
            }
        }
    }
}
