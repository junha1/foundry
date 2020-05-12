// Copyright 2018-2020 Kodebox, Inc.
// This file is part of CodeChain.
//
// This program is free software: you can redistribute it and/or modify
// it under the terms of the GNU Affero General Public License as
// published by the Free Software Foundation, either version 3 of the
// License, or (at your option) any later version.
//
// This program is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU Affero General Public License for more details.
//
// You should have received a copy of the GNU Affero General Public License
// along with this program.  If not, see <https://www.gnu.org/licenses/>.

use ccrypto::blake256;
use ckey::Ed25519Public as Public;
use ctypes::{BlockHash, BlockNumber};
use primitives::{Bytes, H256};
use rlp::Rlp;

/// View onto block header rlp.
pub struct HeaderView<'a> {
    rlp: Rlp<'a>,
}

impl<'a> HeaderView<'a> {
    /// Creates new view onto header from raw bytes.
    pub fn new(bytes: &[u8]) -> HeaderView<'_> {
        HeaderView {
            rlp: Rlp::new(bytes),
        }
    }

    /// Creates new view onto header from rlp.
    pub fn new_from_rlp(rlp: Rlp<'a>) -> HeaderView<'a> {
        HeaderView {
            rlp,
        }
    }

    /// Returns header hash.
    pub fn hash(&self) -> BlockHash {
        blake256(self.rlp.as_raw()).into()
    }

    /// Returns raw rlp.
    pub fn rlp(&self) -> &Rlp<'a> {
        &self.rlp
    }

    /// Returns parent hash.
    pub fn parent_hash(&self) -> BlockHash {
        self.rlp.val_at(0).unwrap()
    }

    /// Returns author.
    pub fn author(&self) -> Public {
        self.rlp.val_at(1).unwrap()
    }

    /// Returns state root.
    pub fn state_root(&self) -> H256 {
        self.rlp.val_at(2).unwrap()
    }

    /// Returns evidences root.
    pub fn evidences_root(&self) -> H256 {
        self.rlp.val_at(3).unwrap()
    }

    /// Returns transactions root.
    pub fn transactions_root(&self) -> H256 {
        self.rlp.val_at(4).unwrap()
    }

    /// Returns next validator set hash
    pub fn next_validator_set_hash(&self) -> H256 {
        self.rlp.val_at(5).unwrap()
    }

    /// Returns block number.
    pub fn number(&self) -> BlockNumber {
        self.rlp.val_at(6).unwrap()
    }

    /// Returns timestamp.
    pub fn timestamp(&self) -> u64 {
        self.rlp.val_at(7).unwrap()
    }

    /// Returns block extra data.
    pub fn extra_data(&self) -> Bytes {
        self.rlp.val_at(8).unwrap()
    }

    /// Returns a vector of post-RLP-encoded seal fields.
    pub fn seal(&self) -> Vec<Bytes> {
        const SIZE_WITHOUT_SEAL: usize = 9;

        let item_count = self.rlp.item_count().unwrap();
        let mut seal = Vec::with_capacity(item_count - SIZE_WITHOUT_SEAL);
        for i in SIZE_WITHOUT_SEAL..item_count {
            seal.push(self.rlp.at(i).unwrap().as_raw().to_vec());
        }
        seal
    }

    /// Returns a vector of seal fields (RLP-decoded).
    pub fn decode_seal(&self) -> Result<Vec<Bytes>, rlp::DecoderError> {
        let seal = self.seal();
        seal.into_iter().map(|s| rlp::Rlp::new(&s).data().map(|x| x.to_vec())).collect()
    }

    /// Get view in the seal field of the header.
    pub fn view(&self) -> u64 {
        let seal = self.seal();
        if let Some(rlp_view) = seal.get(1) {
            Rlp::new(rlp_view.as_slice()).as_val().unwrap()
        } else {
            0
        }
    }
}
