use anyhow::Result;
use num_enum::{IntoPrimitive, TryFromPrimitive};
use std::io::{Read, Write};

mod bitfield;

pub struct PeerMessage {
    id: PeerMessageId,
    payload: Vec<u8>,
}

impl PeerMessage {
    pub fn read(stream: &mut (impl Read + Write)) -> Result<Self> {
        let mut length_buffer = [0_u8; 4];
        stream.read_exact(&mut length_buffer)?;

        let length = u32::from_be_bytes(length_buffer);

        let mut id_buffer = [0_u8; 1];
        stream.read_exact(&mut id_buffer)?;
        let id = u8::from_be_bytes(id_buffer);
        let id = PeerMessageId::try_from(id)?;

        let mut payload = vec![0; length as usize];
        stream.read_exact(&mut payload)?;

        Ok(Self { id, payload })
    }

    pub fn send(&self, stream: &mut (impl Read + Write)) -> Result<()> {
        let mut body = Vec::<u8>::new();
        body.clone_from_slice(&self.payload.len().to_be_bytes());

        let id: u8 = self.id.into();
        body.push(id.to_be());
        body.clone_from_slice(&self.payload);

        stream.write_all(&body)?;
        Ok(())
    }

    pub fn process(&self, stream: &mut (impl Read + Write)) -> Result<()> {
        match self.id {
            PeerMessageId::Bitfield => {
                bitfield::process(self)?;
                let interested_message = PeerMessage {
                    id: PeerMessageId::Interested,
                    payload: Vec::new(),
                };

                interested_message.send(stream)
            }
            PeerMessageId::Unchoke => {
                // TODO: break the desired piece into separate blocks and send a
                // request message for each one.
                todo!();
            }
            PeerMessageId::Piece => {
                todo!();
            }
            _ => Ok(()),
        }
    }
}

#[derive(Copy, Clone, Debug, IntoPrimitive, TryFromPrimitive)]
#[repr(u8)]
pub enum PeerMessageId {
    Choke = 0,
    Unchoke = 1,
    Interested = 2,
    NotInterested = 3,
    Have = 4,
    Bitfield = 5,
    Request = 6,
    Piece = 7,
    Cancel = 8,
}
