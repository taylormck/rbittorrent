use crate::peers::PeerMessage;
use anyhow::Result;

pub fn process(_message: &PeerMessage) -> Result<()> {
    // No-op for now
    Ok(())
}
