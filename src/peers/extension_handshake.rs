use anyhow::Result;
use serde_derive::{Deserialize, Serialize};
use tokio::io::{AsyncRead, AsyncReadExt, AsyncWrite, AsyncWriteExt};

pub async fn shake_hands_extension(
    stream: &mut (impl AsyncRead + AsyncWrite + Unpin),
) -> Result<SupportedExtensions> {
    let dictionary = ExtensionDictionary {
        m: SupportedExtensions::my_supported(),
    };

    let payload = serde_bencode::to_bytes(&dictionary).unwrap();

    let mut handshake = Vec::<u8>::new();
    handshake.extend_from_slice(&u32::to_be_bytes(payload.len() as u32 + 2)[..]);
    handshake.push(u8::to_be(20));
    handshake.push(u8::to_be(0));
    handshake.extend_from_slice(&payload);

    stream.write_all(&handshake).await?;

    let response_size = stream.read_u32().await?;

    let mut response_buffer = vec![0_u8; response_size as usize];
    stream.read_exact(&mut response_buffer).await?;

    // NOTE: We don't need these now, but we can leave this commented out
    // in case we need them in the future.
    // let message_id = u8::from_be(response_buffer[0]);
    // let extension_message_id = u8::from_be(response_buffer[1]);

    let response_dictionary: ExtensionDictionary =
        serde_bencode::from_bytes(&response_buffer[2..])?;

    Ok(response_dictionary.m)
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Eq, Clone, Copy)]
pub struct SupportedExtensions {
    pub ut_metadata: Option<u8>,
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Eq, Clone, Copy)]
struct ExtensionDictionary {
    pub m: SupportedExtensions,
}

impl SupportedExtensions {
    pub fn all_unsupported() -> Self {
        Self { ut_metadata: None }
    }

    pub fn my_supported() -> Self {
        Self {
            ut_metadata: Some(1),
        }
    }
}

#[cfg(test)]
mod test {
    use tokio_test::assert_ok;

    use super::*;

    #[tokio::test]
    async fn test_shake_hands_extension() {
        let dictionary = ExtensionDictionary {
            m: SupportedExtensions::my_supported(),
        };

        let payload = serde_bencode::to_bytes(&dictionary).unwrap();

        let mut handshake = Vec::<u8>::new();
        handshake.extend_from_slice(&u32::to_be_bytes(payload.len() as u32 + 2)[..]);
        handshake.push(u8::to_be(20));
        handshake.push(u8::to_be(0));
        handshake.extend_from_slice(&payload);

        let mut stream = tokio_test::io::Builder::new()
            .write(&handshake.clone())
            .read(&handshake)
            .build();

        let expected_extensions = SupportedExtensions::my_supported();

        let actual_extensions = shake_hands_extension(&mut stream).await;
        assert_ok!(actual_extensions);

        let actual_extensions = actual_extensions.unwrap();
        assert_eq!(expected_extensions, actual_extensions);
    }
}
