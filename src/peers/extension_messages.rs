use anyhow::Result;
use num_enum::{IntoPrimitive, TryFromPrimitive};
use serde_derive::{Deserialize, Serialize};
use std::fmt;
use tokio::io::{AsyncRead, AsyncWrite};

#[derive(Clone, Debug)]
pub struct ExtensionMessage {
    pub id: ExtensionMessageId,
    pub dictionary: MetadataRequestDictionary,
}

impl ExtensionMessage {
    pub fn from_bytes(bytes: &[u8]) -> Result<Self> {
        let id: ExtensionMessageId = u8::from_be(bytes[0]).try_into()?;
        let dictionary = serde_bencode::from_bytes(&bytes[1..])?;

        Ok(Self { id, dictionary })
    }

    pub fn process(&self, _stream: &mut (impl AsyncRead + AsyncWrite + Unpin)) -> Result<()> {
        dbg!(self);
        todo!();
    }
}

#[derive(Copy, Clone, Debug, IntoPrimitive, TryFromPrimitive)]
#[repr(u8)]
pub enum ExtensionMessageId {
    MetadataRequest = 0,
    MetadataResponse = 1,
}

impl fmt::Display for ExtensionMessageId {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}

#[derive(Copy, Clone, Debug, Serialize, Deserialize)]
pub struct MetadataRequestDictionary {
    pub msg_type: u32,
    pub piece: u32,
}
