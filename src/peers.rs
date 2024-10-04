use crate::{IpAddress, Torrent};
use anyhow::Result;
use serde_bencode::value::Value as BValue;
use std::io::Read;

pub fn fetch_peers(torrent: &Torrent) -> Result<Vec<IpAddress>> {
    // This may look scary, but all it does is stick a '%' in between
    // every pair of characters.
    let info_hash = prepare_hash(&torrent.hash);

    let peer_id = "00112233445566778899";
    let port = 6881;
    let uploaded = 0;
    let downloaded = 0;
    let compact = 1;

    // NOTE: We have to manually build the URL like this because if we use reqwest's
    // query builder, it will try to encode the parameters, which breaks the info_hash
    // and peer_id parameters.
    let url = format!(
        "{}?info_hash={}&peer_id={}&port={}&uploaded={}&downloaded={}&left={}&compact={}",
        torrent.announce, info_hash, peer_id, port, uploaded, downloaded, torrent.length, compact
    );

    let mut response = reqwest::blocking::get(url)?;

    if !response.status().is_success() {
        anyhow::bail!("/announce request failed with status {}", response.status());
    }

    let mut body = Vec::<u8>::new();
    response.read_to_end(&mut body)?;

    let body = match serde_bencode::from_bytes::<BValue>(&body) {
        Ok(BValue::Dict(body)) => body,
        _ => anyhow::bail!("Response body is not a bencoded dictionary"),
    };

    let peers = match body.get("peers".as_bytes()) {
        Some(BValue::Bytes(peers)) => peers,
        _ => anyhow::bail!("No peers in response"),
    };

    Ok(peers
        .chunks(6)
        .map(|chunk| {
            let mut peer = [0; 6];
            peer.clone_from_slice(chunk);
            IpAddress::from_bytes(&peer)
        })
        .collect())
}

fn prepare_hash(hash: &str) -> String {
    hash.chars()
        .enumerate()
        .flat_map(|(i, c)| {
            if i % 2 == 0 { Some('%') } else { None }
                .into_iter()
                .chain(std::iter::once(c))
        })
        .collect::<String>()
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    #[test]
    fn test_fetch_peers() {
        let mut server = mockito::Server::new();

        let torrent = Torrent {
            announce: format!("{}/announce", server.url()),
            length: 1337,
            hash: "abcd1234".to_string(),
            piece_length: 0,
            piece_hashes: Vec::<String>::new(),
        };

        let expected_peers = vec![
            IpAddress {
                address: [161, 35, 46, 221],
                port: 51414,
            },
            IpAddress {
                address: [159, 65, 84, 183],
                port: 51444,
            },
            IpAddress {
                address: [167, 172, 57, 188],
                port: 51413,
            },
        ];

        let encoded_expected_peers = expected_peers
            .iter()
            .flat_map(|ip| ip.to_bytes())
            .collect::<Vec<u8>>();

        let mut response_dict = HashMap::new();
        response_dict.insert(
            "peers".as_bytes().to_vec(),
            BValue::Bytes(encoded_expected_peers),
        );

        let response_body = serde_bencode::to_bytes(&BValue::Dict(response_dict)).unwrap();

        let info_hash = prepare_hash(&torrent.hash);
        let peer_id = "00112233445566778899";
        let port = 6881;
        let uploaded = 0;
        let downloaded = 0;
        let compact = 1;

        // NOTE: Because we can't use reqwest's query builder, we have to manually
        // create the full URL in the test as well.
        let url = format!(
            "/announce?info_hash={}&peer_id={}&port={}&uploaded={}&downloaded={}&left={}&compact={}",
            info_hash,
            peer_id,
            port,
            uploaded,
            downloaded,
            torrent.length,
            compact
        );

        let mock = server
            .mock("GET", url.as_str())
            .with_body(response_body)
            .create();

        let actual_peers = fetch_peers(&torrent).unwrap();

        mock.assert();
        assert_eq!(expected_peers, actual_peers);
    }
}
