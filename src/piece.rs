use crate::hash::calculate_hash;
use anyhow::Result;
use std::iter::from_fn;
use tokio::io::{AsyncWrite, AsyncWriteExt};

const BLOCK_SIZE: usize = 16384; // 16 * 1024

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Piece {
    hash: String,
    data: Vec<u8>,
    completed: Vec<bool>,
}

impl Piece {
    pub fn new(length: usize, hash: &str) -> Self {
        let data = vec![0_u8; length];

        let num_blocks = match length % BLOCK_SIZE {
            0 => length / BLOCK_SIZE,
            _ => length / BLOCK_SIZE + 1,
        };

        let completed = vec![false; num_blocks];

        Self {
            data,
            completed,
            hash: hash.to_string(),
        }
    }

    pub fn block_details(&self) -> impl Iterator<Item = (u32, u32)> {
        // NOTE: We copy the length here to avoid borrowing self in the closure.
        let length = self.data.len();
        let mut index = 0;

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
        calculate_hash(&self.data) == self.hash
    }

    pub async fn write(&self, stream: &mut (impl AsyncWrite + Unpin)) -> Result<()> {
        stream.write_all(&self.data).await?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_piece_when_piece_length_divides_block_length_evenly() {
        let length = BLOCK_SIZE * 2;
        let hash = "00112233445566778899".to_string();
        let data = vec![0_u8; length];
        let completed = vec![false; 2];

        let expected_piece = Piece {
            hash: hash.clone(),
            data,
            completed,
        };

        let actual_piece = Piece::new(length, &hash);

        assert_eq!(expected_piece, actual_piece);
    }

    #[test]
    fn test_new_piece_when_piece_length_does_not_divide_block_length_evenly() {
        let length = BLOCK_SIZE * 2 + BLOCK_SIZE / 2;
        let hash = "00112233445566778899".to_string();
        let data = vec![0_u8; length];
        let completed = vec![false; 3];

        let expected_piece = Piece {
            hash: hash.clone(),
            data,
            completed,
        };

        let actual_piece = Piece::new(length, &hash);

        assert_eq!(expected_piece, actual_piece);
    }

    #[test]
    fn test_block_details() {
        let length = BLOCK_SIZE * 2 + BLOCK_SIZE / 2;
        let hash = "00112233445566778899".to_string();
        let piece = Piece::new(length, &hash);
        let mut block_details = piece.block_details();

        assert_eq!((0_u32, BLOCK_SIZE as u32), block_details.next().unwrap());
        assert_eq!(
            (BLOCK_SIZE as u32, BLOCK_SIZE as u32),
            block_details.next().unwrap()
        );
        assert_eq!(
            (BLOCK_SIZE as u32 * 2, BLOCK_SIZE as u32 / 2),
            block_details.next().unwrap()
        );
        assert_eq!(None, block_details.next());
    }

    #[test]
    fn test_update_block() {
        let length = BLOCK_SIZE * 2 + BLOCK_SIZE / 2;
        let hash = "00112233445566778899".to_string();
        let mut actual_piece = Piece::new(length, &hash);

        let mut expected_data = vec![0_u8; length];
        expected_data[BLOCK_SIZE..BLOCK_SIZE * 2].fill(1_u8);
        let expected_completed = vec![false, true, false];

        let expected_piece = Piece {
            hash: hash.clone(),
            data: expected_data,
            completed: expected_completed,
        };

        actual_piece.update_block(BLOCK_SIZE, vec![1_u8; BLOCK_SIZE]);

        assert_eq!(expected_piece, actual_piece);
    }

    #[test]
    fn test_is_complete() {
        let length = BLOCK_SIZE * 2 + BLOCK_SIZE / 2;
        let hash = "00112233445566778899".to_string();
        let mut actual_piece = Piece::new(length, &hash);

        assert!(!actual_piece.is_complete());

        actual_piece.update_block(0, vec![1_u8; BLOCK_SIZE]);
        actual_piece.update_block(BLOCK_SIZE, vec![1_u8; BLOCK_SIZE]);
        actual_piece.update_block(BLOCK_SIZE * 2, vec![1_u8; BLOCK_SIZE / 2]);

        assert!(actual_piece.is_complete());
    }

    #[test]
    fn test_is_valid() {
        let length = BLOCK_SIZE * 2 + BLOCK_SIZE / 2;
        let expected_data = vec![1_u8; length];
        let hash = calculate_hash(&expected_data);

        let mut actual_piece = Piece::new(length, &hash);

        assert!(!actual_piece.is_valid());

        actual_piece.update_block(0, vec![1_u8; BLOCK_SIZE]);
        actual_piece.update_block(BLOCK_SIZE, vec![1_u8; BLOCK_SIZE]);
        actual_piece.update_block(BLOCK_SIZE * 2, vec![1_u8; BLOCK_SIZE / 2]);

        assert!(actual_piece.is_valid());
    }
}
