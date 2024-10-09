use sha1::{Digest, Sha1};

pub fn calculate_hash(input: &[u8]) -> String {
    let mut hasher = Sha1::new();
    hasher.update(input);
    format!("{:x}", hasher.finalize())
}
