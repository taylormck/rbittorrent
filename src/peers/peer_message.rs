use crate::FileInfo;
use anyhow::Result;
use num_enum::{IntoPrimitive, TryFromPrimitive};
use std::fmt;
use tokio::io::{AsyncRead, AsyncReadExt, AsyncWrite, AsyncWriteExt};

#[derive(Clone, Debug)]
pub struct PeerMessage {
    pub id: PeerMessageId,
    payload: Vec<u8>,
}

impl PeerMessage {
    pub fn keep_alive() -> Self {
        Self {
            id: PeerMessageId::KeepAlive,
            payload: vec![],
        }
    }

    pub fn interested() -> Self {
        Self {
            id: PeerMessageId::Interested,
            payload: vec![],
        }
    }

    pub async fn read(stream: &mut (impl AsyncRead + AsyncWrite + Unpin)) -> Result<Self> {
        let length = stream.read_u32().await? as usize;

        if length == 0 {
            return Ok(Self::keep_alive());
        }

        let id: PeerMessageId = stream.read_u8().await?.try_into()?;

        let mut payload = vec![0_u8; length - 1];
        stream.read_exact(&mut payload).await?;

        Ok(Self { id, payload })
    }

    pub async fn send(&self, stream: &mut (impl AsyncRead + AsyncWrite + Unpin)) -> Result<()> {
        let mut body = Vec::<u8>::new();
        let length = self.payload.len() as u32 + 1;
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
            PeerMessageId::Bitfield => Self::interested().send(stream).await?,
            PeerMessageId::Unchoke => {
                let requests = file_info
                    .pieces
                    .iter()
                    .enumerate()
                    .flat_map(|(piece_index, piece)| {
                        let piece_index = piece_index as u32;

                        piece
                            .block_details()
                            .map(move |(block_index, block_size)| {
                                (piece_index, block_index, block_size)
                            })
                            .collect::<Vec<(u32, u32, u32)>>()
                    })
                    .map(|(piece_index, block_index, block_size)| {
                        let mut payload = Vec::<u8>::new();
                        payload.extend_from_slice(&piece_index.to_be_bytes());
                        payload.extend_from_slice(&block_index.to_be_bytes());
                        payload.extend_from_slice(&block_size.to_be_bytes());

                        PeerMessage {
                            id: PeerMessageId::Request,
                            payload,
                        }
                    });

                // TODO: we might want to limit these to ~5 at a time.
                for request in requests {
                    request.send(stream).await?;
                }
            }
            PeerMessageId::Piece => {
                let piece_index = u32::from_be_bytes(self.payload[0..4].try_into()?) as usize;
                let block_index = u32::from_be_bytes(self.payload[4..8].try_into()?) as usize;
                let block_data = self.payload[8..].to_vec();

                file_info.pieces[piece_index].update_block(block_index, block_data);
            }
            PeerMessageId::Extension => {
                todo!();
            }
            PeerMessageId::KeepAlive => {}
            _ => anyhow::bail!("Unimplemented message type: {}", self.id),
        }

        Ok(())
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
    Extension = 20,
    KeepAlive,
}

impl fmt::Display for PeerMessageId {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}
