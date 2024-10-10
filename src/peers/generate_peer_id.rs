use rand::distributions::{Alphanumeric, DistString};

pub fn generate_peer_id() -> String {
    Alphanumeric.sample_string(&mut rand::thread_rng(), 20)
}
