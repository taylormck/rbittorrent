use anyhow::Result;
use num_enum::{IntoPrimitive, TryFromPrimitive};
use std::{collections::HashMap, fmt};

#[derive(Clone, Debug)]
pub struct ExtensionMessage {
    pub peer_extension_id: u8,
    pub payload: HashMap<String, u8>,
}

impl ExtensionMessage {
    pub fn from_bytes(bytes: &[u8]) -> Result<Self> {
        let peer_extension_id = u8::from_be(bytes[0]);
        let payload = serde_bencode::from_bytes(&bytes[1..])?;

        Ok(Self {
            peer_extension_id,
            payload,
        })
    }
}

#[derive(Copy, Clone, Debug, IntoPrimitive, TryFromPrimitive)]
#[repr(u8)]
pub enum ExtensionMessageId {
    Request = 0,
    Data = 1,
    Reject = 2,
}

impl fmt::Display for ExtensionMessageId {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}
