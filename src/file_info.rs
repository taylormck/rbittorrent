use crate::{Piece, Torrent};
use anyhow::Result;
use std::iter::Iterator;
use tokio::fs::File;

#[derive(Clone, Debug)]
pub struct FileInfo {
    path: String,
    pub pieces: Vec<Piece>,
}

impl FileInfo {
    pub fn new(path: String, torrent: &Torrent) -> Self {
        // If our piece size divides evenly into the length, then the last piece
        // will be the same size as the others.
        // Otherwise, the last piece will only be the remaining size.
        let remainder_memory = match torrent.length % torrent.piece_length {
            0 => torrent.piece_length as usize,
            n => n as usize,
        };

        let last_index = torrent.piece_hashes.len() - 1;

        let pieces = torrent
            .piece_hashes
            .iter()
            .enumerate()
            .map(|(i, hash)| {
                let length = match i == last_index {
                    true => remainder_memory,
                    false => torrent.piece_length as usize,
                };

                Piece::new(length, hash)
            })
            .collect();

        Self { path, pieces }
    }

    pub fn is_complete(&self) -> bool {
        self.pieces.iter().all(Piece::is_complete)
    }

    pub fn is_valid(&self) -> bool {
        self.pieces.iter().all(Piece::is_valid)
    }

    pub async fn save_to_disk(&self) -> Result<()> {
        if !self.is_complete() {
            anyhow::bail!("Not all file pieces are complete!");
        }

        if !self.is_valid() {
            anyhow::bail!("Not all file pieces are valid!");
        }

        let mut file = File::create(&self.path).await?;

        for piece in &self.pieces {
            if let Err(err) = piece.write(&mut file).await {
                anyhow::bail!("Error writing file: {}", err);
            }
        }

        Ok(())
    }
}
