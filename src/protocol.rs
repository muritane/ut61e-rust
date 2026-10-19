#[derive(Debug)]
pub struct PacketParser {
    buffer: Vec<u8>,
    expected_length: Option<usize>,
}

impl PacketParser {
    pub fn new() -> Self {
        Self {
            buffer: Vec::new(),
            expected_length: None,
        }
    }

    pub fn push(&mut self, byte: u8) -> Option<Vec<u8>> {
        // Waiting for first header byte
        if self.buffer.is_empty() {
            if byte == 0xAB {
                self.buffer.push(byte);
            }
            return None;
        }

        // Waiting for second header byte
        if self.buffer.len() == 1 {
            if byte == 0xCD {
                self.buffer.push(byte);
            } else {
                self.buffer.clear();

                // A new AB could start another packet
                if byte == 0xAB {
                    self.buffer.push(byte);
                }
            }

            return None;
        }

        // Waiting for length byte
        if self.buffer.len() == 2 {
            let length = byte as usize;

            // Reject impossible or suspicious lengths.
            if !(2..=64).contains(&length) {
                self.buffer.clear();
                self.expected_length = None;
                return None;
            }

            self.buffer.push(byte);

            // Header(2) + length field (1) + following bytes.
            self.expected_length = Some(3 + length);

            return None;
        }

        self.buffer.push(byte);

        if let Some(expected) = self.expected_length {
            if self.buffer.len() == expected {
                let packet = std::mem::take(&mut self.buffer);
                self.expected_length = None;

                let checksum_index = packet.len() - 2;

                let calculated: u16 = packet[..checksum_index]
                    .iter()
                    .map(|&b| b as u16)
                    .sum();

                let received = u16::from_be_bytes([
                    packet[checksum_index],
                    packet[checksum_index + 1],
                ]);

                if calculated != received {
                    eprintln!(
                        "Checksum mismatch: calculated {calculated:04X}, received {received:04X}"
                    );
                    return None;
                }

                // Return only the payload.
                return Some(packet[3..checksum_index].to_vec());
            }
        }

        None
    }
}
