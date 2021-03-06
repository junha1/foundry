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

use super::super::errors;
use super::super::traits::Chain;
use super::super::types::{Block, BlockNumberAndHash, Transaction, ValidatorSet};
use ccore::{BlockChainClient, EngineInfo};
use ckey::{NetworkId, PlatformAddress};
use ctypes::{BlockHash, BlockId, BlockNumber, TxHash};
use jsonrpc_core::Result;
use std::sync::Arc;

pub struct ChainClient<C>
where
    C: BlockChainClient + EngineInfo, {
    client: Arc<C>,
}

impl<C> ChainClient<C>
where
    C: BlockChainClient + EngineInfo,
{
    pub fn new(client: Arc<C>) -> Self {
        ChainClient {
            client,
        }
    }
}

impl<C> Chain for ChainClient<C>
where
    C: BlockChainClient + EngineInfo + 'static,
{
    fn get_transaction(&self, transaction_hash: TxHash) -> Result<Option<Transaction>> {
        let id = transaction_hash.into();
        Ok(self.client.transaction(&id).map(From::from))
    }

    fn contains_transaction(&self, transaction_hash: TxHash) -> Result<bool> {
        Ok(self.client.transaction_block(&transaction_hash.into()).is_some())
    }

    fn get_best_block_number(&self) -> Result<BlockNumber> {
        Ok(self.client.chain_info().best_block_number)
    }

    fn get_best_block_id(&self) -> Result<BlockNumberAndHash> {
        let chain_info = self.client.chain_info();
        Ok(BlockNumberAndHash {
            number: chain_info.best_block_number,
            hash: chain_info.best_block_hash,
        })
    }

    fn get_block_hash(&self, block_number: u64) -> Result<Option<BlockHash>> {
        Ok(self.client.block_hash(&BlockId::Number(block_number)))
    }

    fn get_block_by_number(&self, block_number: u64) -> Result<Option<Block>> {
        let id = BlockId::Number(block_number);
        Ok(self.client.block(&id).map(|block| Block::from_core(block.decode(), self.client.network_id())))
    }

    fn get_block_by_hash(&self, block_hash: BlockHash) -> Result<Option<Block>> {
        let id = BlockId::Hash(block_hash);
        Ok(self.client.block(&id).map(|block| {
            let block = block.decode();
            Block::from_core(block, self.client.network_id())
        }))
    }

    fn get_block_transaction_count_by_hash(&self, block_hash: BlockHash) -> Result<Option<usize>> {
        Ok(self.client.block(&BlockId::Hash(block_hash)).map(|block| block.transactions_count()))
    }

    fn get_network_id(&self) -> Result<NetworkId> {
        Ok(self.client.network_id())
    }

    fn get_possible_authors(&self, block_number: Option<u64>) -> Result<Option<Vec<PlatformAddress>>> {
        Ok(self.client.possible_authors(block_number).map_err(errors::core)?)
    }

    fn get_validator_set(&self, block_number: Option<u64>) -> Result<Option<ValidatorSet>> {
        let validator_set_in_core = self.client.validator_set(block_number).map_err(errors::core)?;
        Ok(validator_set_in_core.map(ValidatorSet::from_core))
    }
}
