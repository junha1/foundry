// Copyright 2018-2020 Kodebox, Inc.
// This file is part of CodeChain.
//
// This program is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as
// published by the Free Software Foundation, either version 3 of the
// License, or (at your option) any later version.
//
// This program is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.
//
// You should have received a copy of the GNU General Public License
// along with this program.  If not, see <https://www.gnu.org/licenses/>.

use super::MessageID;
use ccore::Evidence;
use coordinator::Transaction;
use ctypes::SyncHeader;
use rlp::{DecoderError, Encodable, Rlp, RlpStream};

#[derive(Debug)]
pub enum ResponseMessage {
    Headers(Vec<SyncHeader>),
    Bodies(Vec<(Vec<Evidence>, Vec<Transaction>)>),
    StateChunk(Vec<Vec<u8>>),
}

impl Encodable for ResponseMessage {
    fn rlp_append(&self, s: &mut RlpStream) {
        match self {
            ResponseMessage::Headers(headers) => {
                s.append_list(headers);
            }
            ResponseMessage::Bodies(bodies) => {
                s.begin_list(1);

                let uncompressed = {
                    let mut inner_list = RlpStream::new_list(bodies.len());
                    bodies.iter().for_each(|(evidences, transactions)| {
                        inner_list.begin_list(2);
                        inner_list.append_list(evidences);
                        inner_list.append_list(transactions);
                    });
                    inner_list.out()
                };

                let compressed = {
                    // TODO: Cache the Encoder object
                    let mut snappy_encoder = snap::Encoder::new();
                    snappy_encoder.compress_vec(&uncompressed).expect("Compression always succeed")
                };

                s.append(&compressed);
            }
            ResponseMessage::StateChunk(chunks) => {
                s.append_list::<Vec<u8>, Vec<u8>>(chunks);
            }
        };
    }
}

impl ResponseMessage {
    pub fn message_id(&self) -> MessageID {
        match self {
            ResponseMessage::Headers {
                ..
            } => MessageID::Headers,
            ResponseMessage::Bodies(..) => MessageID::Bodies,
            ResponseMessage::StateChunk {
                ..
            } => MessageID::StateChunk,
        }
    }

    pub fn decode(id: MessageID, rlp: &Rlp<'_>) -> Result<Self, DecoderError> {
        let message = match id {
            MessageID::Headers => ResponseMessage::Headers(rlp.as_list()?),
            MessageID::Bodies => {
                let item_count = rlp.item_count()?;
                if item_count != 1 {
                    return Err(DecoderError::RlpIncorrectListLen {
                        got: item_count,
                        expected: 1,
                    })
                }

                let compressed: Vec<u8> = rlp.val_at(0)?;
                let uncompressed = {
                    // TODO: Cache the Decoder object
                    let mut snappy_decoder = snap::Decoder::new();
                    snappy_decoder.decompress_vec(&compressed).map_err(|err| {
                        cwarn!(SYNC, "Decompression failed while decoding a body response: {}", err);
                        DecoderError::Custom("Invalid compression format")
                    })?
                };

                let uncompressed_rlp = Rlp::new(&uncompressed);

                let mut bodies = Vec::new();
                for item in uncompressed_rlp.into_iter() {
                    let evidences = item.list_at(0)?;
                    let transactions = item.list_at(1)?;
                    bodies.push((evidences, transactions));
                }
                ResponseMessage::Bodies(bodies)
            }
            MessageID::StateChunk => ResponseMessage::StateChunk(rlp.as_list()?),
            _ => return Err(DecoderError::Custom("Unknown message id detected")),
        };

        Ok(message)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use coordinator::Transaction;
    use ctypes::Header;
    use rlp::{Encodable, Rlp};

    pub fn decode_bytes(id: MessageID, bytes: &[u8]) -> ResponseMessage {
        let rlp = Rlp::new(bytes);
        ResponseMessage::decode(id, &rlp).unwrap()
    }

    /// For a type that does not have PartialEq, uses Debug instead.
    fn assert_eq_by_debug<T: std::fmt::Debug>(a: &T, b: &T) {
        assert_eq!(format!("{:?}", a), format!("{:?}", b));
    }

    #[test]
    fn headers_message_rlp() {
        let headers = vec![SyncHeader::new(Header::default(), None)];
        headers.iter().for_each(|header| {
            header.hash();
        });

        let message = ResponseMessage::Headers(headers);
        assert_eq_by_debug(&message, &decode_bytes(message.message_id(), message.rlp_bytes().as_ref()))
    }

    #[test]
    fn bodies_message_rlp() {
        let message = ResponseMessage::Bodies(vec![(vec![], vec![])]);
        assert_eq_by_debug(&message, &decode_bytes(message.message_id(), message.rlp_bytes().as_ref()));

        let tx = Transaction::new("sample".to_string(), vec![1, 2, 3, 4, 5]);

        let message = ResponseMessage::Bodies(vec![(vec![], vec![tx])]);
        assert_eq_by_debug(&message, &decode_bytes(message.message_id(), message.rlp_bytes().as_ref()));
    }

    #[test]
    fn state_chunk_message_rlp() {
        let message = ResponseMessage::StateChunk(vec![]);
        assert_eq_by_debug(&message, &decode_bytes(message.message_id(), message.rlp_bytes().as_ref()));
    }
}
