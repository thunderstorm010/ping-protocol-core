use smallvec::SmallVec;

use crate::{OwnedProtocolMessage, HEADER};

#[derive(Debug, PartialEq, Clone)]
pub enum ParseError {
    InvalidStartByte,
    IncompleteData,
    ChecksumError(OwnedProtocolMessage),
}

#[derive(Debug)]
pub enum DecoderResult {
    Success(OwnedProtocolMessage),
    InProgress,
    Error(ParseError),
}

#[derive(Debug)]
pub enum DecoderState {
    AwaitingStart1,
    AwaitingStart2,
    ReadingHeader { buf: [u8; 6], idx: u8 },
    ReadingPayload { buf: SmallVec<[u8; 128]> },
    ReadingChecksum { buf: [u8; 2], idx: u8 },
}

pub struct Decoder {
    pub state: DecoderState,
    message: OwnedProtocolMessage,
}

impl Decoder {
    pub fn new() -> Self {
        Self {
            state: DecoderState::AwaitingStart1,
            message: OwnedProtocolMessage::default(),
        }
    }

    pub fn parse_byte(&mut self, byte: u8) -> DecoderResult {
        match self.state {
            DecoderState::AwaitingStart1 => {
                if byte == HEADER[0] {
                    self.state = DecoderState::AwaitingStart2;
                    return DecoderResult::InProgress;
                }
                return DecoderResult::Error(ParseError::InvalidStartByte);
            }
            DecoderState::AwaitingStart2 => {
                if byte == HEADER[1] {
                    self.state = DecoderState::ReadingHeader {
                        buf: [0; 6],
                        idx: 0,
                    };
                    return DecoderResult::InProgress;
                }
                self.state = DecoderState::AwaitingStart1;
                return DecoderResult::Error(ParseError::InvalidStartByte);
            }
            DecoderState::ReadingHeader {
                ref mut buf,
                mut idx,
            } => {
                buf[idx as usize] = byte;
                idx += 1;
                // Basic information is available, moving to payload state
                if idx == 6 {
                    self.message.payload_length = u16::from_le_bytes([buf[0], buf[1]]);
                    self.message.message_id = u16::from_le_bytes([buf[2], buf[3]]);
                    self.message.src_device_id = buf[4];
                    self.message.dst_device_id = buf[5];

                    if self.message.payload_length == 0 {
                        self.state = DecoderState::ReadingChecksum {
                            buf: [0; 2],
                            idx: 0,
                        }
                    } else {
                        self.state = DecoderState::ReadingPayload {
                            buf: SmallVec::with_capacity(self.message.payload_length as usize),
                        };
                    }
                }
                return DecoderResult::InProgress;
            }
            DecoderState::ReadingPayload { ref mut buf } => {
                buf.push(byte);
                if buf.len() == self.message.payload_length as usize {
                    let state = core::mem::replace(
                        &mut self.state,
                        DecoderState::ReadingChecksum {
                            buf: [0; 2],
                            idx: 0,
                        },
                    );
                    let DecoderState::ReadingPayload { buf } = state else {
                        unreachable!()
                    };
                    self.message.payload = buf;
                }
                return DecoderResult::InProgress;
            }
            DecoderState::ReadingChecksum { mut buf, mut idx } => {
                buf[idx as usize] = byte;
                idx += 1;
                if idx == 2 {
                    self.message.checksum = u16::from_le_bytes([buf[0], buf[1]]);
                    self.reset();
                    let message =
                        core::mem::replace(&mut self.message, OwnedProtocolMessage::default());

                    if !message.as_protocol_message().has_valid_crc() {
                        return DecoderResult::Error(ParseError::ChecksumError(message));
                    }
                    return DecoderResult::Success(message);
                }
                return DecoderResult::InProgress;
            }
        }
    }

    fn reset(&mut self) {
        self.state = DecoderState::AwaitingStart1;
    }
}
