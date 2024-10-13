use anyhow::Result;
use serde_derive::{Deserialize, Serialize};
use tokio::io::{AsyncRead, AsyncWrite, AsyncWriteExt};

pub async fn shake_hands_extension(
    stream: &mut (impl AsyncRead + AsyncWrite + Unpin),
) -> Result<()> {
    let dictionary = ExtensionDictionary {
        m: SupportedExtensions { ut_metadata: 1 },
    };

    let payload = serde_bencode::to_bytes(&dictionary).unwrap();

    let mut handshake = Vec::<u8>::new();
    handshake.extend_from_slice(&u32::to_be_bytes(payload.len() as u32 + 2)[..]);
    handshake.push(u8::to_be(20));
    handshake.push(u8::to_be(0));
    handshake.extend_from_slice(&payload);

    stream.write_all(&handshake).await?;

    Ok(())
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Eq, Clone, Copy)]
struct SupportedExtensions {
    pub ut_metadata: u8,
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Eq, Clone, Copy)]
struct ExtensionDictionary {
    pub m: SupportedExtensions,
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Eq, Clone)]
pub struct SupportedExtensionIds {
    metadata: Option<String>,
}

#[cfg(test)]
mod test {
    use tokio_test::assert_ok;

    use super::*;

    #[tokio::test]
    async fn test_shake_hands_extension() {
        let dictionary = ExtensionDictionary {
            m: SupportedExtensions { ut_metadata: 1 },
        };

        let payload = serde_bencode::to_bytes(&dictionary).unwrap();

        let mut handshake = Vec::<u8>::new();
        handshake.extend_from_slice(&u32::to_be_bytes(payload.len() as u32 + 2)[..]);
        handshake.push(u8::to_be(20));
        handshake.push(u8::to_be(0));
        handshake.extend_from_slice(&payload);

        let mut stream = tokio_test::io::Builder::new()
            .write(&handshake.clone())
            .build();

        assert_ok!(shake_hands_extension(&mut stream).await);
    }
}
