use anyhow::Result;
use num_enum::{IntoPrimitive, TryFromPrimitive};
use tokio::io::{AsyncRead, AsyncReadExt, AsyncWrite, AsyncWriteExt};

mod bitfield;

#[derive(Clone, Debug)]
pub struct PeerMessage {
    id: PeerMessageId,
    payload: Vec<u8>,
}

impl PeerMessage {
    pub async fn read(stream: &mut (impl AsyncRead + AsyncWrite + Unpin)) -> Result<Self> {
        let mut length_buffer = [0_u8; 4];
        stream.read_exact(&mut length_buffer).await?;

        let length = u32::from_be_bytes(length_buffer);

        let mut id_buffer = [0_u8; 1];
        stream.read_exact(&mut id_buffer).await?;
        let id = u8::from_be_bytes(id_buffer);
        let id = PeerMessageId::try_from(id)?;

        let mut payload = vec![0; length as usize];
        stream.read_exact(&mut payload).await?;

        Ok(Self { id, payload })
    }

    pub async fn send(&self, stream: &mut (impl AsyncRead + AsyncWrite + Unpin)) -> Result<()> {
        dbg!("Sending message: {:?}", self);

        let mut body = Vec::<u8>::new();
        let length = self.payload.len() as u32;
        body.extend_from_slice(&length.to_be_bytes());

        let id: u8 = self.id.into();
        body.push(id.to_be());
        body.extend_from_slice(&self.payload);

        Ok(stream.write_all(&body).await?)
    }

    pub async fn process(&self, stream: &mut (impl AsyncRead + AsyncWrite + Unpin)) -> Result<()> {
        match self.id {
            PeerMessageId::Bitfield => {
                bitfield::process(self)?;
                let interested_message = PeerMessage {
                    id: PeerMessageId::Interested,
                    payload: Vec::new(),
                };

                interested_message.send(stream).await
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
