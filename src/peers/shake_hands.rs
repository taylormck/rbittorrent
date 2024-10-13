use crate::Torrent;
use anyhow::Result;
use bitflags::bitflags;
use core::fmt;
use std::marker::Unpin;
use tokio::io::{AsyncRead, AsyncReadExt, AsyncWrite, AsyncWriteExt};

pub async fn shake_hands(
    stream: &mut (impl AsyncRead + AsyncWrite + Unpin),
    torrent: &Torrent,
    peer_id: &str,
    reserved_bytes: HandshakeReservedBytes,
) -> Result<HandshakeResponse> {
    let mut handshake = vec![u8::to_be(19)];

    // Standard header
    handshake.extend_from_slice(b"BitTorrent protocol");

    // Reserved bytes
    handshake.extend_from_slice(&reserved_bytes.bits().to_be_bytes());

    // Hash
    let hash = hex::decode(&torrent.hash)?;
    handshake.extend_from_slice(&hash);

    // Peer ID
    handshake.extend_from_slice(peer_id.as_bytes());

    let mut buffer = [0_u8; 68];

    match stream.write(&handshake).await {
        Ok(68) => {}
        Ok(num) => anyhow::bail!("Sent {} bytes, expected 68.", num),
        Err(err) => anyhow::bail!(err),
    }

    match stream.read(&mut buffer).await {
        Ok(68) => {}
        Ok(num) => anyhow::bail!("Received {} bytes, expected 68.", num),
        Err(err) => anyhow::bail!(err),
    }

    let encoded_peer_id = hex::encode(&buffer[48..68]);
    let reserved_bytes =
        HandshakeReservedBytes::from_bits_truncate(u64::from_be_bytes(buffer[20..28].try_into()?));

    Ok(HandshakeResponse {
        encoded_peer_id,
        reserved_bytes,
    })
}

bitflags! {
    #[derive(Clone, Copy, Debug, PartialEq, Eq)]
    pub struct HandshakeReservedBytes: u64 {
        const ExtensionsEnabled = 1 << 20;
    }
}

impl fmt::Display for HandshakeReservedBytes {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:64b}", self.bits())
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct HandshakeResponse {
    pub encoded_peer_id: String,
    pub reserved_bytes: HandshakeReservedBytes,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_shake_hands() {
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

        let expected_response = HandshakeResponse {
            encoded_peer_id: hex::encode(peer_id),
            reserved_bytes: HandshakeReservedBytes::empty(),
        };

        let mut stream = tokio_test::io::Builder::new()
            .read(&handshake.clone())
            .write(&handshake)
            .build();

        let actual_response = shake_hands(
            &mut stream,
            &torrent,
            peer_id,
            HandshakeReservedBytes::empty(),
        )
        .await
        .unwrap();

        assert_eq!(expected_response, actual_response);
    }

    #[tokio::test]
    async fn test_shake_hands_with_extensions() {
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
        handshake.extend_from_slice(
            &HandshakeReservedBytes::ExtensionsEnabled
                .bits()
                .to_be_bytes(),
        );

        // Hash
        let hash = hex::decode(&torrent.hash).unwrap();
        handshake.extend_from_slice(&hash);

        // Peer ID
        let peer_id = "00112233445566778899";
        handshake.extend_from_slice(peer_id.as_bytes());

        let expected_response = HandshakeResponse {
            encoded_peer_id: hex::encode(peer_id),
            reserved_bytes: HandshakeReservedBytes::ExtensionsEnabled,
        };

        let mut stream = tokio_test::io::Builder::new()
            .read(&handshake.clone())
            .write(&handshake)
            .build();

        let actual_response = shake_hands(
            &mut stream,
            &torrent,
            peer_id,
            HandshakeReservedBytes::ExtensionsEnabled,
        )
        .await
        .unwrap();

        assert_eq!(expected_response, actual_response);
    }
}
