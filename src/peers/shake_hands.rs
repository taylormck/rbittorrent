use crate::Torrent;
use anyhow::Result;
use std::io::{Read, Write};

pub fn shake_hands(stream: &mut (impl Read + Write), torrent: &Torrent) -> Result<String> {
    let mut handshake = Vec::<u8>::new();

    // Standard header
    handshake.push(u8::to_be(19));
    handshake.extend_from_slice(b"BitTorrent protocol");

    // Placeholder bytes
    handshake.extend_from_slice(&[0_u8; 8]);

    // Hash
    let hash = hex::decode(&torrent.hash)?;
    handshake.extend_from_slice(&hash);

    // Peer ID
    handshake.extend_from_slice("00112233445566778899".as_bytes());

    let mut buffer = [0_u8; 68];

    match stream.write(&handshake) {
        Ok(68) => {}
        Ok(num) => anyhow::bail!("Sent {} bytes, expected 68", num),
        Err(err) => anyhow::bail!(err),
    }

    match stream.read(&mut buffer) {
        Ok(68) => {}
        Ok(num) => anyhow::bail!("Received {} bytes, expected 68", num),
        Err(err) => anyhow::bail!(err),
    }

    Ok(hex::encode(&buffer[48..68]))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::VecDeque;

    #[test]
    fn test_shake_hands() {
        let torrent = Torrent {
            announce: "fake-url/announce".to_string(),
            length: 1337,
            hash: hex::encode("12345678901234567890"),
            piece_length: 0,
            piece_hashes: Vec::<String>::new(),
        };

        let mut handshake = Vec::<u8>::new();

        // Standard header
        handshake.push(u8::to_be(19));
        handshake.extend_from_slice(b"BitTorrent protocol");

        // Placeholder bytes
        handshake.extend_from_slice(&[0_u8; 8]);

        // Hash
        let hash = hex::decode(&torrent.hash).unwrap();
        handshake.extend_from_slice(&hash);

        // Peer ID
        let peer_id = "00112233445566778899";
        handshake.extend_from_slice(peer_id.as_bytes());

        let mut stream = VecDeque::<u8>::from(handshake);

        let result = shake_hands(&mut stream, &torrent).unwrap();

        assert_eq!(result, hex::encode(peer_id));
    }
}
