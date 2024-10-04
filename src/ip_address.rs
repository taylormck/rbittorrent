use std::fmt;

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct IpAddress {
    pub address: [u8; 4],
    pub port: u16,
}

impl IpAddress {
    pub fn from_bytes(bytes: &[u8; 6]) -> Self {
        Self {
            address: bytes[0..4]
                .iter()
                .map(|b| u8::from_be_bytes([*b]))
                .collect::<Vec<u8>>()
                .try_into()
                .unwrap(),
            port: u16::from_be_bytes(bytes[4..6].try_into().unwrap()),
        }
    }

    pub fn to_bytes(&self) -> [u8; 6] {
        let mut bytes = [0; 6];
        bytes[0] = self.address[0].to_be();
        bytes[1] = self.address[1].to_be();
        bytes[2] = self.address[2].to_be();
        bytes[3] = self.address[3].to_be();
        bytes[4..6].clone_from_slice(&self.port.to_be_bytes());

        bytes
    }
}

impl fmt::Display for IpAddress {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{}.{}.{}.{}:{}",
            self.address[0], self.address[1], self.address[2], self.address[3], self.port
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_from_bytes() {
        let input_bytes: [u8; 6] = [0x0a, 0x0a, 0x00, 0x01, 0x00, 0x16];

        let expected_ip = IpAddress {
            address: [10, 10, 0, 1],
            port: 22,
        };

        let actual_ip = IpAddress::from_bytes(&input_bytes);

        assert_eq!(expected_ip, actual_ip);
    }

    #[test]
    fn test_to_bytes() {
        let input_ip = IpAddress {
            address: [10, 10, 0, 1],
            port: 22,
        };

        let expected_bytes: [u8; 6] = [0x0a, 0x0a, 0x00, 0x01, 0x00, 0x16];
        let actual_bytes = input_ip.to_bytes();

        assert_eq!(expected_bytes, actual_bytes);
    }

    #[test]
    fn test_inverse() {
        let input_bytes: [u8; 6] = [0x0a, 0x0a, 0x00, 0x01, 0x00, 0x16];

        let ip = IpAddress::from_bytes(&input_bytes);
        let output_bytes = ip.to_bytes();

        assert_eq!(input_bytes, output_bytes);
    }
}
