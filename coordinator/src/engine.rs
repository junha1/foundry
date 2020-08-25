// Copyright 2020 Kodebox, Inc.
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

use crate::context::StorageAccess;
use crate::header::Header;
use crate::transaction::{Transaction, TransactionWithMetadata};
use crate::types::{BlockOutcome, CloseBlockError, ErrorCode, HeaderError, TransactionOutcome, VerifiedCrime};
use ctypes::{CompactValidatorSet, ConsensusParams};
use parking_lot::Mutex;
use std::sync::Arc;

pub trait Initializer: Send + Sync {
    fn initialize_chain(&self, storage: Arc<Mutex<dyn StorageAccess>>) -> (CompactValidatorSet, ConsensusParams);
}

pub trait BlockExecutor: Send + Sync {
    fn open_block(
        &self,
        storage: Arc<Mutex<dyn StorageAccess>>,
        header: &Header,
        verified_crimes: &[VerifiedCrime],
    ) -> Result<(), HeaderError>;
    fn execute_transactions(&self, transactions: &[Transaction]) -> Result<Vec<TransactionOutcome>, ()>;
    fn prepare_block<'a>(
        &self,
        transactions: &mut dyn Iterator<Item = &'a TransactionWithMetadata>,
    ) -> Vec<(&'a Transaction, TransactionOutcome)>;
    fn close_block(&self) -> Result<BlockOutcome, CloseBlockError>;
}

pub trait TxFilter: Send + Sync {
    fn check_transaction(&self, transaction: &Transaction) -> Result<(), ErrorCode>;
    fn filter_transactions<'a>(
        &self,
        transactions: &mut dyn Iterator<Item = &'a TransactionWithMetadata>,
        memory_limit: Option<usize>,
        size_limit: Option<usize>,
    ) -> FilteredTxs<'a>;
}

pub struct FilteredTxs<'a> {
    pub invalid: Vec<&'a Transaction>,
    pub low_priority: Vec<&'a Transaction>,
}

pub trait GraphQlHandlerProvider: Send + Sync {
    /// Returns list of (module name, module graphql handler).
    fn get(&self) -> Vec<(String, Arc<dyn super::module::HandleGraphQlRequest>)>;
}
