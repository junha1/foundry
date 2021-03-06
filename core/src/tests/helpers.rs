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

use coordinator::test_coordinator::TestCoordinator;
use ctypes::{BlockHash, Header};
use primitives::Bytes;
use rlp::RlpStream;

use crate::genesis::Genesis;

pub fn create_test_block(header: &Header) -> Bytes {
    let mut rlp = RlpStream::new_list(3);
    rlp.append(header);
    rlp.append_raw(&rlp::EMPTY_LIST_RLP, 1); // evidences
    rlp.append_raw(&rlp::EMPTY_LIST_RLP, 1);
    rlp.out()
}

pub fn get_good_dummy_block() -> Bytes {
    let (_, bytes) = get_good_dummy_block_hash();
    bytes
}

pub fn get_good_dummy_block_hash() -> (BlockHash, Bytes) {
    let mut block_header = Header::new();
    let genesis = Genesis::new(Default::default(), &TestCoordinator::default());
    block_header.set_timestamp(40);
    block_header.set_number(1);
    block_header.set_parent_hash(genesis.header().hash());

    (block_header.hash(), create_test_block(&block_header))
}
