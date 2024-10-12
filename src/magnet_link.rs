use anyhow::Result;
use std::str::FromStr;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct MagnetLink {
    pub hash: String,
    pub file_name: String,
    pub tracker_url: String,
}

impl FromStr for MagnetLink {
    type Err = anyhow::Error;

    fn from_str(link: &str) -> Result<Self> {
        if !link.starts_with("magnet:?") {
            anyhow::bail!("Not a magnet link");
        }

        let mut hash: Option<String> = None;
        let mut file_name: Option<String> = None;
        let mut tracker_url: Option<String> = None;

        let pairs = link[8..].split('&').flat_map(|s| s.split_once('='));

        for (key, value) in pairs {
            match key {
                "xt" => {
                    if !value.starts_with("urn:btih:") {
                        anyhow::bail!("Invalid hash");
                    }
                    hash = Some(value[9..].to_owned());
                }
                "dn" => file_name = Some(value.to_owned()),
                "tr" => tracker_url = Some(urlencoding::decode(value)?.into_owned()),
                _ => anyhow::bail!("Invalid key: {}", key),
            }
        }

        if hash.is_none() {
            anyhow::bail!("No hash found");
        }

        Ok(Self {
            hash: hash.unwrap(),
            file_name: file_name.unwrap_or("".to_string()),
            tracker_url: tracker_url.unwrap_or("".to_string()),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_from_str() {
        let input = "magnet:?\
            xt=urn:btih:d69f91e6b2ae4c542468d1073a71d4ea13879a7f&\
            dn=sample.torrent&\
            tr=http%3A%2F%2Fbittorrent-test-tracker.codecrafters.io%2Fannounce";

        let expected = MagnetLink {
            hash: "d69f91e6b2ae4c542468d1073a71d4ea13879a7f".to_string(),
            file_name: "sample.torrent".to_string(),
            tracker_url: "http://bittorrent-test-tracker.codecrafters.io/announce".to_string(),
        };

        assert_eq!(expected, input.parse().unwrap());
    }
}
