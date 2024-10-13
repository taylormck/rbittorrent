use anyhow::Result;
use serde_derive::{Deserialize, Serialize};
use tokio::io::{AsyncRead, AsyncWrite, AsyncWriteExt};

pub async fn shake_hands_extension(
    stream: &mut (impl AsyncRead + AsyncWrite + Unpin),
) -> Result<()> {
    let mut handshake = vec![u8::to_be(20), u8::to_be(0)];

    let dictionary = ExtensionDictionary {
        m: SupportedExtensions { metadata: 1 },
    };

    let payload = serde_bencode::to_bytes(&dictionary).unwrap();

    handshake.extend_from_slice(&payload);

    let expected_bytes = handshake.len();

    match stream.write(&handshake).await {
        Ok(sent_bytes) if sent_bytes == expected_bytes => Ok(()),
        Ok(sent_bytes) => anyhow::bail!("Sent {} bytes, expected {}.", sent_bytes, expected_bytes),
        Err(err) => anyhow::bail!(err),
    }
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Eq, Clone, Copy)]
struct SupportedExtensions {
    pub metadata: u8,
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Eq, Clone, Copy)]
struct ExtensionDictionary {
    pub m: SupportedExtensions,
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Eq, Clone)]
pub struct SupportedExtensionIds {
    metadata: Option<String>,
}
