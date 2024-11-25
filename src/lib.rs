#![no_std]

use smallvec::SmallVec;
extern crate alloc;
pub mod decoder;

pub const HEADER: [u8; 2] = ['B' as u8, 'R' as u8];

#[derive(Debug, Clone, PartialEq)]
pub struct ProtocolMessage<'a> {
    pub payload_length: u16,
    pub message_id: u16,
    pub src_device_id: u8,
    pub dst_device_id: u8,
    pub payload: &'a [u8],
    pub checksum: u16,
}

#[derive(Debug, Clone, PartialEq, Default)]
pub struct OwnedProtocolMessage {
    pub payload_length: u16,
    pub message_id: u16,
    pub src_device_id: u8,
    pub dst_device_id: u8,
    pub payload: SmallVec<[u8; 128]>,
    pub checksum: u16,
}

impl OwnedProtocolMessage {
    pub fn as_protocol_message(&self) -> ProtocolMessage {
        ProtocolMessage {
            payload_length: self.payload_length,
            message_id: self.message_id,
            src_device_id: self.src_device_id,
            dst_device_id: self.dst_device_id,
            payload: &self.payload,
            checksum: self.checksum
        }
    }
}

impl<'a> Default for ProtocolMessage<'a> {
    fn default() -> Self {
        Self {
            payload_length: Default::default(),
            message_id: Default::default(),
            src_device_id: Default::default(),
            dst_device_id: Default::default(),
            payload: Default::default(),
            checksum: Default::default(),
        }
    }
}

impl<'a> ProtocolMessage<'a> {
    pub async fn serialize_async<O: embedded_io_async::Write>(
        &self,
        out: &mut O,
    ) -> Result<(), O::Error> {
        out.write_all(&HEADER).await?;
        out.write_all(&self.payload_length.to_le_bytes()).await?;
        out.write_all(&self.message_id.to_le_bytes()).await?;
        out.write_all(&self.src_device_id.to_le_bytes()).await?;
        out.write_all(&self.dst_device_id.to_le_bytes()).await?;
        out.write_all(&self.payload).await?;
        out.write_all(&self.checksum.to_le_bytes()).await?;
        Ok(())
    }

    pub fn calculate_crc(&self) -> u16 {
        let mut checksum: u16 = 0;
        checksum = checksum.wrapping_add(HEADER[0] as u16);
        checksum = checksum.wrapping_add(HEADER[1] as u16);
        self.payload_length
            .to_le_bytes()
            .iter()
            .for_each(|byte| checksum = checksum.wrapping_add(*byte as u16));
        self.message_id
            .to_le_bytes()
            .iter()
            .for_each(|byte| checksum = checksum.wrapping_add(*byte as u16));
        checksum = checksum.wrapping_add(self.src_device_id as u16);
        checksum = checksum.wrapping_add(self.dst_device_id as u16);
        for &byte in self.payload.iter() {
            checksum = checksum.wrapping_add(byte as u16);
        }
        checksum
    }

    pub fn has_valid_crc(&self) -> bool {
        self.checksum == self.calculate_crc()
    }

    pub fn length(&self) -> usize {
        HEADER.len() + 2 + 2 + 1 + 1 + self.payload_length as usize + 2
    }
}
