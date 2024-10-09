use crate::{calculate_hash, Torrent};
use anyhow::Result;
use std::iter::{from_fn, Iterator};
use tokio::{fs::File, io::AsyncWriteExt};

const BLOCK_SIZE: usize = 16384; // 16 * 1024

pub struct FileInfo {
    path: String,
    pub pieces: Vec<Piece>,
}

impl FileInfo {
    pub fn new(path: String, torrent: &Torrent) -> Self {
        let pieces = torrent
            .piece_hashes
            .iter()
            .map(|hash| Piece::new(torrent.piece_length as usize, hash))
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
        let mut file = File::create(&self.path).await?;

        if !self.is_complete() {
            anyhow::bail!("Not all file pieces are complete!");
        }

        if !self.is_valid() {
            anyhow::bail!("Not all file pieces are valid!");
        }

        if let Err(err) = file.write_all(&self.pieces[0].data).await {
            anyhow::bail!("Error writing file: {}", err);
        }

        Ok(())
    }
}

pub struct Piece {
    hash: String,
    data: Vec<u8>,
    completed: Vec<bool>,
}

impl Piece {
    pub fn new(length: usize, hash: &str) -> Self {
        let data = Vec::with_capacity(length);
        let num_blocks = length / BLOCK_SIZE;
        let completed = vec![false; num_blocks];

        Self {
            data,
            completed,
            hash: hash.to_string(),
        }
    }

    pub fn block_details(&self) -> impl Iterator<Item = (u32, u32)> {
        let mut index = 0;

        // NOTE: We copy the length here to avoid borrowing self in the closure.
        let length = self.data.len();

        from_fn(move || {
            let result;

            if index < length {
                let block_size = usize::min(BLOCK_SIZE, length - index);
                result = Some((index as u32, block_size as u32));
                index += block_size;
            } else {
                result = None;
            }

            result
        })
    }

    pub fn update_block(&mut self, index: usize, data: Vec<u8>) {
        self.data.splice(index..index + data.len(), data);

        let completed_index = index / BLOCK_SIZE;
        self.completed[completed_index] = true;
    }

    pub fn is_complete(&self) -> bool {
        self.completed.iter().all(|b| *b)
    }

    pub fn is_valid(&self) -> bool {
        // calculate_hash(&self.data) == self.hash
        true
    }
}
