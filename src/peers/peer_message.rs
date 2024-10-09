use crate::FileInfo;
use anyhow::Result;
use num_enum::{IntoPrimitive, TryFromPrimitive};
use tokio::io::{AsyncRead, AsyncReadExt, AsyncWrite, AsyncWriteExt};

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

    pub async fn process(
        &self,
        stream: &mut (impl AsyncRead + AsyncWrite + Unpin),
        file_info: &mut FileInfo,
    ) -> Result<()> {
        match self.id {
            PeerMessageId::Bitfield => {
                let interested_message = PeerMessage {
                    id: PeerMessageId::Interested,
                    payload: Vec::new(),
                };

                interested_message.send(stream).await
            }
            PeerMessageId::Unchoke => {
                // TODO: we might want to limit these to ~5 at a time.
                for piece in &file_info.pieces {
                    for (i, block) in piece.block_details().enumerate() {
                        let mut payload = Vec::<u8>::new();
                        payload.extend_from_slice(&(i as u32).to_be_bytes());
                        payload.extend_from_slice(&block.0.to_be_bytes());
                        payload.extend_from_slice(&block.1.to_be_bytes());

                        let request_message = PeerMessage {
                            id: PeerMessageId::Request,
                            payload,
                        };

                        if let Err(err) = request_message.send(stream).await {
                            anyhow::bail!("Error sending request message: {}", err);
                        }
                    }
                }

                Ok(())
            }
            PeerMessageId::Piece => {
                let piece_index = u32::from_be_bytes(self.payload[0..4].try_into()?) as usize;
                let begin_index = u32::from_be_bytes(self.payload[4..8].try_into()?) as usize;
                let block = self.payload[8..].to_vec();

                file_info.pieces[piece_index].update_block(begin_index, block);

                Ok(())
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
