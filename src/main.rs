use bittorrent_starter_rust::{
    bencode, calculate_hash,
    peers::{
        self, generate_peer_id, ExtensionMessage, HandshakeReservedBytes, PeerMessage,
        PeerMessageId,
    },
    FileInfo, MagnetLink, Torrent,
};
use clap::{Parser, Subcommand};
use serde_bencode::value::Value as BValue;
use std::collections::HashMap;
use tokio::{fs::File, net::TcpStream};

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
    Download {
        #[arg(short, long = "out")]
        output_path: Option<String>,
        file_path: String,
    },
    MagnetParse {
        magnet_link: String,
    },
    MagnetHandshake {
        magnet_link: String,
    },
    MagnetInfo {
        magnet_link: String,
    },
    MagnetDownloadPiece {
        #[arg(short, long = "out")]
        output_path: Option<String>,
        magnet_link: String,
        piece_index: usize,
    },
    MagnetDownload {
        #[arg(short, long = "out")]
        output_path: Option<String>,
        magnet_link: String,
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
            let peer_id = generate_peer_id();
            let torrent_peers = peers::fetch_peers(&torrent, &peer_id).unwrap();

            torrent_peers.iter().for_each(|peer| println!("{}", peer));
        }
        Commands::Handshake { file_path, peer_ip } => {
            let torrent = Torrent::from_file(file_path).unwrap();
            let peer_id = generate_peer_id();

            let mut stream = match TcpStream::connect(peer_ip).await {
                Ok(stream) => stream,
                Err(err) => {
                    eprintln!("Error connecting to peer: {}", err);
                    std::process::exit(1);
                }
            };

            match peers::shake_hands(
                &mut stream,
                &torrent,
                &peer_id,
                HandshakeReservedBytes::empty(),
            )
            .await
            {
                Ok(result) => println!("Peer ID: {}", result.encoded_peer_id),
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
            let peer_id = generate_peer_id();

            if *piece_index >= torrent.piece_hashes.len() {
                eprintln!("Invalid piece index");
                std::process::exit(1);
            }

            let torrent_peers = peers::fetch_peers(&torrent, &peer_id).unwrap();
            let peer_ip = torrent_peers[0];

            let output_path = output_path.clone().unwrap_or("/tmp/output".to_string());
            let mut file_info = FileInfo::new(output_path.clone(), &torrent);

            let mut stream = match TcpStream::connect(peer_ip).await {
                Ok(stream) => stream,
                Err(err) => {
                    eprintln!("Error connecting to peer: {}", err);
                    std::process::exit(1);
                }
            };

            if let Err(err) = peers::shake_hands(
                &mut stream,
                &torrent,
                &peer_id,
                HandshakeReservedBytes::empty(),
            )
            .await
            {
                eprintln!("Error shaking hands: {}", err);
                std::process::exit(1);
            }

            loop {
                let message = PeerMessage::read(&mut stream).await;

                if let Err(err) = message {
                    eprintln!("Error reading message: {}", err);
                    std::process::exit(1);
                }

                let message = message.unwrap();

                if let Err(err) = message.process(&mut stream, &mut file_info).await {
                    eprintln!("Error processing message: {}", err);
                    std::process::exit(1);
                }

                // TODO: probably a more performant way to handle this
                if file_info.is_complete() {
                    break;
                }
            }

            let piece = &file_info.pieces[*piece_index];

            if !piece.is_complete() {
                eprintln!("Piece is not complete");
                std::process::exit(1);
            }

            if !piece.is_valid() {
                eprintln!("Piece is not valid");
                std::process::exit(1);
            }

            let mut file = File::create(output_path).await.unwrap();

            if let Err(err) = piece.write(&mut file).await {
                eprintln!("Unable to save file to disk: {}", err);
                std::process::exit(1);
            }
        }
        Commands::Download {
            output_path,
            file_path,
        } => {
            let torrent = Torrent::from_file(file_path).unwrap();
            let peer_id = generate_peer_id();

            let torrent_peers = peers::fetch_peers(&torrent, &peer_id).unwrap();
            let peer_ip = torrent_peers[0];

            let output_path = output_path.clone().unwrap_or("/tmp/output".to_string());
            let mut file_info = FileInfo::new(output_path.clone(), &torrent);

            let mut stream = match TcpStream::connect(peer_ip).await {
                Ok(stream) => stream,
                Err(err) => {
                    eprintln!("Error connecting to peer: {}", err);
                    std::process::exit(1);
                }
            };

            if let Err(err) = peers::shake_hands(
                &mut stream,
                &torrent,
                &peer_id,
                HandshakeReservedBytes::empty(),
            )
            .await
            {
                eprintln!("Error shaking hands: {}", err);
                std::process::exit(1);
            }

            loop {
                let message = PeerMessage::read(&mut stream).await;

                if let Err(err) = message {
                    eprintln!("Error reading message: {}", err);
                    std::process::exit(1);
                }

                let message = message.unwrap();

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
        Commands::MagnetParse { magnet_link } => {
            let magnet_link: MagnetLink = magnet_link.parse().unwrap();

            println!("Tracker URL: {}", magnet_link.tracker_url);
            println!("Info Hash: {}", magnet_link.hash);
        }
        Commands::MagnetHandshake { magnet_link } => {
            let magnet_link: MagnetLink = magnet_link.parse().unwrap();
            let placeholder_torrent = Torrent {
                hash: magnet_link.hash,
                announce: magnet_link.tracker_url,
                length: 999, // fake length to make the peer happy
                ..Default::default()
            };

            let peer_id = generate_peer_id();
            let peers = peers::fetch_peers(&placeholder_torrent, &peer_id).unwrap();
            let peer_ip = peers[0];

            let mut stream = match TcpStream::connect(peer_ip).await {
                Ok(stream) => stream,
                Err(err) => {
                    eprintln!("Error connecting to peer: {}", err);
                    std::process::exit(1);
                }
            };

            let base_handshake_result = match peers::shake_hands(
                &mut stream,
                &placeholder_torrent,
                &peer_id,
                HandshakeReservedBytes::ExtensionsEnabled,
            )
            .await
            {
                Ok(base_handshake_result) => base_handshake_result,
                Err(err) => {
                    eprintln!("Error shaking hands: {}", err);
                    std::process::exit(1);
                }
            };

            println!("Peer ID: {}", base_handshake_result.encoded_peer_id);

            // Recieve the bitfield message
            match PeerMessage::read(&mut stream).await {
                Ok(message) => match message.id {
                    PeerMessageId::Bitfield => {}
                    _ => {
                        eprintln!("Unexpected message: {:?}", message);
                        std::process::exit(1);
                    }
                },
                Err(err) => {
                    eprintln!("Error reading bitfield message: {}", err);
                    std::process::exit(1);
                }
            }

            if !base_handshake_result
                .reserved_bytes
                .contains(HandshakeReservedBytes::ExtensionsEnabled)
            {
                return;
            }

            let extensions = match peers::shake_hands_extension(&mut stream).await {
                Ok(extensions) => extensions,
                Err(err) => {
                    eprintln!("Error shaking hands for extensions: {}", err);
                    std::process::exit(1);
                }
            };

            if let Some(ut_metadata) = extensions.ut_metadata {
                println!("Peer Metadata Extension ID: {}", ut_metadata);
            }
        }
        Commands::MagnetInfo { magnet_link } => {
            let magnet_link: MagnetLink = magnet_link.parse().unwrap();
            let placeholder_torrent = Torrent {
                hash: magnet_link.hash,
                announce: magnet_link.tracker_url.clone(),
                length: 999, // fake length to make the peer happy
                ..Default::default()
            };

            let peer_id = generate_peer_id();
            let peers = peers::fetch_peers(&placeholder_torrent, &peer_id).unwrap();
            let peer_ip = peers[0];

            let mut stream = match TcpStream::connect(peer_ip).await {
                Ok(stream) => stream,
                Err(err) => {
                    eprintln!("Error connecting to peer: {}", err);
                    std::process::exit(1);
                }
            };

            let base_handshake_result = match peers::shake_hands(
                &mut stream,
                &placeholder_torrent,
                &peer_id,
                HandshakeReservedBytes::ExtensionsEnabled,
            )
            .await
            {
                Ok(base_handshake_result) => base_handshake_result,
                Err(err) => {
                    eprintln!("Error shaking hands: {}", err);
                    std::process::exit(1);
                }
            };

            // Recieve the bitfield message
            match PeerMessage::read(&mut stream).await {
                Ok(message) => match message.id {
                    PeerMessageId::Bitfield => {}
                    _ => {
                        eprintln!("Unexpected message: {:?}", message);
                        std::process::exit(1);
                    }
                },
                Err(err) => {
                    eprintln!("Error reading bitfield message: {}", err);
                    std::process::exit(1);
                }
            }

            if !base_handshake_result
                .reserved_bytes
                .contains(HandshakeReservedBytes::ExtensionsEnabled)
            {
                return;
            }

            let extensions = match peers::shake_hands_extension(&mut stream).await {
                Ok(extensions) => extensions,
                Err(err) => {
                    eprintln!("Error shaking hands for extensions: {}", err);
                    std::process::exit(1);
                }
            };

            if let Some(peer_ut_metadata_id) = extensions.ut_metadata {
                let dictionary =
                    HashMap::from([("msg_type".to_string(), 0_u8), ("piece".to_string(), 0_u8)]);

                let mut payload = vec![peer_ut_metadata_id];
                payload.extend_from_slice(&serde_bencode::to_bytes(&dictionary).unwrap());

                let request_message = PeerMessage {
                    id: PeerMessageId::Extension,
                    payload,
                };

                if let Err(err) = request_message.send(&mut stream).await {
                    eprintln!("Error sending request: {}", err);
                    std::process::exit(1);
                }

                if let Ok(message) = PeerMessage::read(&mut stream).await {
                    let message = match message.id {
                        PeerMessageId::Extension => message,
                        _ => {
                            eprintln!("Unexpected message: {:?}", message);
                            std::process::exit(1);
                        }
                    };

                    let extension_message = ExtensionMessage::from_bytes(&message.payload).unwrap();
                    let metadata_length =
                        *extension_message.payload.get("total_size").unwrap() as usize;

                    let start_index = message.payload.len() - metadata_length;
                    let metadata = match serde_bencode::from_bytes(&message.payload[start_index..])
                    {
                        Ok(BValue::Dict(dict)) => dict,
                        _ => {
                            eprintln!("Invalid metadata");
                            std::process::exit(1);
                        }
                    };

                    let length = match metadata.get("length".as_bytes()) {
                        Some(BValue::Int(len)) => *len,
                        _ => panic!("Torrent file does not contain length entry."),
                    };

                    let piece_length = match metadata.get("piece length".as_bytes()) {
                        Some(BValue::Int(len)) => *len,
                        _ => panic!("Torrent file does not contain piece length entry."),
                    };

                    let pieces = match metadata.get("pieces".as_bytes()) {
                        Some(BValue::Bytes(bytes)) => bytes,
                        _ => panic!("Torrent file does not contain pieces entry."),
                    };

                    let piece_hashes = pieces.chunks(20).map(const_hex::encode).collect();

                    let hash = calculate_hash(&message.payload[start_index..]);

                    let torrent = Torrent {
                        announce: magnet_link.tracker_url,
                        length,
                        piece_length,
                        piece_hashes,
                        hash,
                    };

                    println!("Tracker URL: {}", torrent.announce);
                    println!("Length: {}", torrent.length);
                    println!("Info Hash: {}", torrent.hash);
                    println!("Piece Length: {}", torrent.piece_length);
                    println!("Piece Hashes: \n{}", torrent.piece_hashes.join("\n"));
                }
            }
        }
        Commands::MagnetDownloadPiece {
            magnet_link,
            output_path,
            piece_index,
        } => {
            let magnet_link: MagnetLink = magnet_link.parse().unwrap();
            let placeholder_torrent = Torrent {
                hash: magnet_link.hash,
                announce: magnet_link.tracker_url.clone(),
                length: 999, // fake length to make the peer happy
                ..Default::default()
            };

            let peer_id = generate_peer_id();
            let peers = peers::fetch_peers(&placeholder_torrent, &peer_id).unwrap();
            let peer_ip = peers[0];

            let mut stream = match TcpStream::connect(peer_ip).await {
                Ok(stream) => stream,
                Err(err) => {
                    eprintln!("Error connecting to peer: {}", err);
                    std::process::exit(1);
                }
            };

            let base_handshake_result = match peers::shake_hands(
                &mut stream,
                &placeholder_torrent,
                &peer_id,
                HandshakeReservedBytes::ExtensionsEnabled,
            )
            .await
            {
                Ok(base_handshake_result) => base_handshake_result,
                Err(err) => {
                    eprintln!("Error shaking hands: {}", err);
                    std::process::exit(1);
                }
            };

            // Recieve the bitfield message
            match PeerMessage::read(&mut stream).await {
                Ok(message) => match message.id {
                    PeerMessageId::Bitfield => {}
                    _ => {
                        eprintln!("Unexpected message: {:?}", message);
                        std::process::exit(1);
                    }
                },
                Err(err) => {
                    eprintln!("Error reading bitfield message: {}", err);
                    std::process::exit(1);
                }
            }

            if !base_handshake_result
                .reserved_bytes
                .contains(HandshakeReservedBytes::ExtensionsEnabled)
            {
                return;
            }

            let extensions = match peers::shake_hands_extension(&mut stream).await {
                Ok(extensions) => extensions,
                Err(err) => {
                    eprintln!("Error shaking hands for extensions: {}", err);
                    std::process::exit(1);
                }
            };

            if let Some(peer_ut_metadata_id) = extensions.ut_metadata {
                let dictionary =
                    HashMap::from([("msg_type".to_string(), 0_u8), ("piece".to_string(), 0_u8)]);

                let mut payload = vec![peer_ut_metadata_id];
                payload.extend_from_slice(&serde_bencode::to_bytes(&dictionary).unwrap());

                let request_message = PeerMessage {
                    id: PeerMessageId::Extension,
                    payload,
                };

                if let Err(err) = request_message.send(&mut stream).await {
                    eprintln!("Error sending request: {}", err);
                    std::process::exit(1);
                }

                if let Ok(message) = PeerMessage::read(&mut stream).await {
                    let message = match message.id {
                        PeerMessageId::Extension => message,
                        _ => {
                            eprintln!("Unexpected message: {:?}", message);
                            std::process::exit(1);
                        }
                    };

                    let extension_message = ExtensionMessage::from_bytes(&message.payload).unwrap();
                    let metadata_length =
                        *extension_message.payload.get("total_size").unwrap() as usize;

                    let start_index = message.payload.len() - metadata_length;
                    let metadata = match serde_bencode::from_bytes(&message.payload[start_index..])
                    {
                        Ok(BValue::Dict(dict)) => dict,
                        _ => {
                            eprintln!("Invalid metadata");
                            std::process::exit(1);
                        }
                    };

                    let length = match metadata.get("length".as_bytes()) {
                        Some(BValue::Int(len)) => *len,
                        _ => panic!("Torrent file does not contain length entry."),
                    };

                    let piece_length = match metadata.get("piece length".as_bytes()) {
                        Some(BValue::Int(len)) => *len,
                        _ => panic!("Torrent file does not contain piece length entry."),
                    };

                    let pieces = match metadata.get("pieces".as_bytes()) {
                        Some(BValue::Bytes(bytes)) => bytes,
                        _ => panic!("Torrent file does not contain pieces entry."),
                    };

                    let piece_hashes = pieces.chunks(20).map(const_hex::encode).collect();

                    let hash = calculate_hash(&message.payload[start_index..]);

                    let torrent = Torrent {
                        announce: magnet_link.tracker_url,
                        length,
                        piece_length,
                        piece_hashes,
                        hash,
                    };

                    if *piece_index >= torrent.piece_hashes.len() {
                        eprintln!("Invalid piece index");
                        std::process::exit(1);
                    }

                    let output_path = output_path.clone().unwrap_or("/tmp/output".to_string());
                    let mut file_info = FileInfo::new(output_path.clone(), &torrent);

                    if let Err(err) = PeerMessage::interested().send(&mut stream).await {
                        eprintln!("Error sending interested: {}", err);
                        std::process::exit(1);
                    }

                    loop {
                        let message = PeerMessage::read(&mut stream).await;

                        if let Err(err) = message {
                            eprintln!("Error reading message: {}", err);
                            std::process::exit(1);
                        }

                        let message = message.unwrap();

                        if let Err(err) = message.process(&mut stream, &mut file_info).await {
                            eprintln!("Error processing message: {}", err);
                            std::process::exit(1);
                        }

                        if file_info.is_complete() {
                            break;
                        }
                    }

                    let piece = &file_info.pieces[*piece_index];

                    if !piece.is_complete() {
                        eprintln!("Piece is not complete");
                        std::process::exit(1);
                    }

                    if !piece.is_valid() {
                        eprintln!("Piece is not valid");
                        std::process::exit(1);
                    }

                    let mut file = File::create(output_path).await.unwrap();

                    if let Err(err) = piece.write(&mut file).await {
                        eprintln!("Unable to save file to disk: {}", err);
                        std::process::exit(1);
                    }
                }
            }
        }
        Commands::MagnetDownload {
            magnet_link,
            output_path,
        } => {
            let magnet_link: MagnetLink = magnet_link.parse().unwrap();
            let placeholder_torrent = Torrent {
                hash: magnet_link.hash,
                announce: magnet_link.tracker_url.clone(),
                length: 999, // fake length to make the peer happy
                ..Default::default()
            };

            let peer_id = generate_peer_id();
            let peers = peers::fetch_peers(&placeholder_torrent, &peer_id).unwrap();
            let peer_ip = peers[0];

            let mut stream = match TcpStream::connect(peer_ip).await {
                Ok(stream) => stream,
                Err(err) => {
                    eprintln!("Error connecting to peer: {}", err);
                    std::process::exit(1);
                }
            };

            let base_handshake_result = match peers::shake_hands(
                &mut stream,
                &placeholder_torrent,
                &peer_id,
                HandshakeReservedBytes::ExtensionsEnabled,
            )
            .await
            {
                Ok(base_handshake_result) => base_handshake_result,
                Err(err) => {
                    eprintln!("Error shaking hands: {}", err);
                    std::process::exit(1);
                }
            };

            // Recieve the bitfield message
            match PeerMessage::read(&mut stream).await {
                Ok(message) => match message.id {
                    PeerMessageId::Bitfield => {}
                    _ => {
                        eprintln!("Unexpected message: {:?}", message);
                        std::process::exit(1);
                    }
                },
                Err(err) => {
                    eprintln!("Error reading bitfield message: {}", err);
                    std::process::exit(1);
                }
            }

            if !base_handshake_result
                .reserved_bytes
                .contains(HandshakeReservedBytes::ExtensionsEnabled)
            {
                return;
            }

            let extensions = match peers::shake_hands_extension(&mut stream).await {
                Ok(extensions) => extensions,
                Err(err) => {
                    eprintln!("Error shaking hands for extensions: {}", err);
                    std::process::exit(1);
                }
            };

            if let Some(peer_ut_metadata_id) = extensions.ut_metadata {
                let dictionary =
                    HashMap::from([("msg_type".to_string(), 0_u8), ("piece".to_string(), 0_u8)]);

                let mut payload = vec![peer_ut_metadata_id];
                payload.extend_from_slice(&serde_bencode::to_bytes(&dictionary).unwrap());

                let request_message = PeerMessage {
                    id: PeerMessageId::Extension,
                    payload,
                };

                if let Err(err) = request_message.send(&mut stream).await {
                    eprintln!("Error sending request: {}", err);
                    std::process::exit(1);
                }

                if let Ok(message) = PeerMessage::read(&mut stream).await {
                    let message = match message.id {
                        PeerMessageId::Extension => message,
                        _ => {
                            eprintln!("Unexpected message: {:?}", message);
                            std::process::exit(1);
                        }
                    };

                    let extension_message = ExtensionMessage::from_bytes(&message.payload).unwrap();
                    let metadata_length =
                        *extension_message.payload.get("total_size").unwrap() as usize;

                    let start_index = message.payload.len() - metadata_length;
                    let metadata = match serde_bencode::from_bytes(&message.payload[start_index..])
                    {
                        Ok(BValue::Dict(dict)) => dict,
                        _ => {
                            eprintln!("Invalid metadata");
                            std::process::exit(1);
                        }
                    };

                    let length = match metadata.get("length".as_bytes()) {
                        Some(BValue::Int(len)) => *len,
                        _ => panic!("Torrent file does not contain length entry."),
                    };

                    let piece_length = match metadata.get("piece length".as_bytes()) {
                        Some(BValue::Int(len)) => *len,
                        _ => panic!("Torrent file does not contain piece length entry."),
                    };

                    let pieces = match metadata.get("pieces".as_bytes()) {
                        Some(BValue::Bytes(bytes)) => bytes,
                        _ => panic!("Torrent file does not contain pieces entry."),
                    };

                    let piece_hashes = pieces.chunks(20).map(const_hex::encode).collect();

                    let hash = calculate_hash(&message.payload[start_index..]);

                    let torrent = Torrent {
                        announce: magnet_link.tracker_url,
                        length,
                        piece_length,
                        piece_hashes,
                        hash,
                    };

                    let output_path = output_path.clone().unwrap_or("/tmp/output".to_string());
                    let mut file_info = FileInfo::new(output_path.clone(), &torrent);

                    if let Err(err) = PeerMessage::interested().send(&mut stream).await {
                        eprintln!("Error sending interested: {}", err);
                        std::process::exit(1);
                    }

                    loop {
                        let message = PeerMessage::read(&mut stream).await;

                        if let Err(err) = message {
                            eprintln!("Error reading message: {}", err);
                            std::process::exit(1);
                        }

                        let message = message.unwrap();

                        if let Err(err) = message.process(&mut stream, &mut file_info).await {
                            eprintln!("Error processing message: {}", err);
                            std::process::exit(1);
                        }

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
    }
}
