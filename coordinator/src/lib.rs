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

#[macro_use]
extern crate codechain_logger as clogger;
#[macro_use]
extern crate log;

#[macro_use]
mod desc_common;
pub mod app_desc;
pub mod context;
pub mod engine;
mod header;
mod link_desc;
mod linkable;
pub mod module;
pub mod test_coordinator;
mod transaction;
pub mod types;
pub mod values;
mod weaver;

pub use crate::app_desc::AppDesc;
use crate::context::StorageAccess;
use crate::engine::{BlockExecutor, ExecutionId, GraphQlHandlerProvider, Initializer, TxFilter};
pub use crate::header::Header;
pub use crate::link_desc::LinkDesc;
use crate::module::{
    HandleCrimes, HandleGraphQlRequest, InitConsensus, InitGenesis, SessionId, SortedTxs, Stateful, TxOwner, TxSorter,
    UpdateConsensus,
};
pub use crate::transaction::{Transaction, TransactionWithMetadata, TxOrigin};
use crate::types::{
    BlockOutcome, CloseBlockError, ErrorCode, ExecuteTransactionError, FilteredTxs, HeaderError, TransactionOutcome,
    VerifiedCrime,
};
use crate::weaver::Weaver;
use cmodule::sandbox::Sandbox;
use ctypes::StorageId;
use ctypes::{ChainParams, CompactValidatorSet};
use parking_lot::{Mutex, RwLock};
use remote_trait_object::{Service, ServiceRef};
use std::collections::HashMap;
use std::mem;
use std::ops::Bound;
use std::ops::Bound::*;
use std::sync::Arc;

pub(crate) const HOST_ID: &str = "$";

pub(crate) const TX_SERVICES_FOR_HOST: &[&str] = &["tx-owner"];

pub(crate) type Occurrences = (Bound<usize>, Bound<usize>);

pub(crate) static SERVICES_FOR_HOST: &[(Occurrences, &str)] = &[
    ((Included(0), Unbounded), "init-genesis"),
    ((Included(1), Excluded(2)), "init-consensus"),
    ((Included(0), Excluded(2)), "update-consensus"),
    ((Included(0), Unbounded), "stateful"),
    ((Included(0), Excluded(2)), "tx-sorter"),
    ((Included(0), Excluded(2)), "handle-crimes"),
    ((Included(0), Unbounded), "handle-graphql-request"),
];

type SessionSlot = u128;

/// The `Coordinator` encapsulates all the logic for a Foundry application.
///
/// It assembles modules and feeds them various events from the underlying
/// consensus engine.
pub struct Coordinator {
    /// Currently active sessions represented as bits set.
    sessions: RwLock<Vec<SessionSlot>>,

    /// The key services from modules for implementing a chain.
    services: Services,

    /// List of `Sandbox`es of the modules constituting the current application.
    _sandboxes: Vec<Box<dyn Sandbox>>,
}

const SESSION_BITS_PER_SLOT: usize = mem::size_of::<SessionSlot>() * 8;

impl Coordinator {
    pub fn from_descs(app_desc: &AppDesc, link_desc: &LinkDesc) -> anyhow::Result<Coordinator> {
        cmodule::init_modules();

        let weaver = Weaver::new();
        let (sandboxes, mut services) = weaver.weave(app_desc, link_desc)?;

        // The order of stateful decides the assignment of substorage ids. It MUST be deterministic.
        services.stateful.lock().sort_by(|a, b| a.0.cmp(&b.0));

        services.genesis_config = app_desc
            .modules
            .iter()
            .map(|(name, setup)| ((**name).clone(), serde_cbor::to_vec(&setup.genesis_config).unwrap()))
            .collect();

        Ok(Coordinator {
            services,
            _sandboxes: sandboxes,
            sessions: RwLock::new(vec![0]),
        })
    }

    fn new_session(&self, storage: &mut dyn StorageAccess) -> SessionId {
        let session_id = {
            let mut sessions = self.sessions.write();
            let (index, bit) = sessions
                .iter()
                .enumerate()
                .find_map(|(i, &bits)| {
                    if bits == SessionSlot::MAX {
                        None
                    } else {
                        Some((i, bits.trailing_ones()))
                    }
                })
                .unwrap_or_else(|| {
                    sessions.push(0);
                    (sessions.len() - 1, 0)
                });

            sessions[index] |= 1 << bit;
            bit + (SESSION_BITS_PER_SLOT * index) as SessionId
        };

        let mut statefuls = self.services.stateful.lock();
        for (storage_id, (_, stateful)) in statefuls.iter_mut().enumerate() {
            let sub_storage = storage.sub_storage(storage_id as StorageId);
            stateful.new_session(session_id, ServiceRef::create_export(sub_storage));
        }
        session_id
    }

    fn end_session(&self, session_id: SessionId) {
        let mut statefuls = self.services.stateful.lock();
        for (_, ref mut stateful) in statefuls.iter_mut() {
            stateful.end_session(session_id);
        }
        let mut sessions = self.sessions.write();
        let session_id = session_id as usize;
        sessions[session_id / SESSION_BITS_PER_SLOT] &= !(1 << (session_id % SESSION_BITS_PER_SLOT));
    }

    pub fn services(&self) -> &Services {
        &self.services
    }
}

pub struct Services {
    /// List of module name and `Stateful` service pairs in the current app.
    /// The module name is used to keep the index of the corresponding `Stateful`
    /// same across updates, since the index is used as `StorageId`.
    pub stateful: Mutex<Vec<(String, Box<dyn Stateful>)>>,

    /// List of module name and its `InitGenesis` pairs.
    pub init_genesis: Vec<(String, Box<dyn InitGenesis>)>,

    /// Per-module genesis config.
    pub genesis_config: HashMap<String, Vec<u8>>,

    /// A map from Tx type to its owner.
    pub tx_owner: HashMap<String, Box<dyn TxOwner>>,

    /// An optional crime handler.
    pub handle_crimes: Box<dyn HandleCrimes>,

    /// A service responsible for initializing the validators and the parameters.
    pub init_consensus: Box<dyn InitConsensus>,

    /// A service responsible for updating the validators and the parameters when closing every block.
    pub update_consensus: Box<dyn UpdateConsensus>,

    /// A service sorting Tx'es in the mempool.
    pub tx_sorter: Box<dyn TxSorter>,

    /// A map from module name to its GraphQL handler
    pub handle_graphqls: Vec<(String, Arc<dyn HandleGraphQlRequest>)>,
}

impl Default for Services {
    fn default() -> Self {
        Self {
            stateful: Mutex::new(Vec::new()),
            init_genesis: Vec::new(),
            genesis_config: Default::default(),
            tx_owner: Default::default(),
            handle_crimes: Box::new(NoOpHandleCrimes) as Box<dyn HandleCrimes>,
            init_consensus: Box::new(PanickingInitConsensus) as Box<dyn InitConsensus>,
            update_consensus: Box::new(NoOpUpdateConsensus) as Box<dyn UpdateConsensus>,
            tx_sorter: Box::new(DefaultTxSorter) as Box<dyn TxSorter>,
            handle_graphqls: Default::default(),
        }
    }
}

struct NoOpHandleCrimes;

impl Service for NoOpHandleCrimes {}

impl HandleCrimes for NoOpHandleCrimes {
    fn handle_crimes(&self, _session_id: SessionId, _crimes: &[VerifiedCrime]) {}
}

struct PanickingInitConsensus;

impl Service for PanickingInitConsensus {}

impl InitConsensus for PanickingInitConsensus {
    fn init_consensus(&self, _session_id: SessionId) -> (CompactValidatorSet, ChainParams) {
        panic!("There must be a `InitConsensus` service")
    }
}

struct NoOpUpdateConsensus;

impl Service for NoOpUpdateConsensus {}

impl UpdateConsensus for NoOpUpdateConsensus {
    fn update_consensus(&self, _session_id: SessionId) -> (Option<CompactValidatorSet>, Option<ChainParams>) {
        (None, None)
    }
}

struct DefaultTxSorter;

impl Service for DefaultTxSorter {}

impl TxSorter for DefaultTxSorter {
    fn sort_txs(&self, _session_id: SessionId, txs: &[TransactionWithMetadata]) -> SortedTxs {
        SortedTxs {
            invalid: Vec::new(),
            sorted: (0..txs.len()).collect(),
        }
    }
}

impl Initializer for Coordinator {
    fn number_of_sub_storages(&self) -> usize {
        self.services.stateful.lock().len()
    }

    fn initialize_chain(&self, storage: &mut dyn StorageAccess) -> (CompactValidatorSet, ChainParams) {
        let services = &self.services;
        let session_id = self.new_session(storage);

        for (ref module, ref init) in services.init_genesis.iter() {
            let config = match services.genesis_config.get(module) {
                Some(value) => value as &[u8],
                None => &[],
            };
            init.init_genesis(session_id, config);
        }

        let (validator_set, params) = services.init_consensus.init_consensus(session_id);

        self.end_session(session_id);

        (validator_set, params)
    }
}

impl BlockExecutor for Coordinator {
    fn open_block(
        &self,
        storage: &mut dyn StorageAccess,
        header: &Header,
        verified_crimes: &[VerifiedCrime],
    ) -> Result<ExecutionId, HeaderError> {
        cdebug!(COORDINATOR, "open block");
        let services = &self.services;

        let session_id = self.new_session(storage);

        services.handle_crimes.handle_crimes(session_id, verified_crimes);

        for owner in services.tx_owner.values() {
            owner.block_opened(session_id, header)?;
        }

        Ok(session_id)
    }

    fn execute_transactions(
        &self,
        execution_id: ExecutionId,
        storage: &mut dyn StorageAccess,
        transactions: &[Transaction],
    ) -> Result<Vec<TransactionOutcome>, ExecuteTransactionError> {
        let services = &self.services;

        let mut outcomes = Vec::with_capacity(transactions.len());
        let session_id = execution_id as SessionId;

        for tx in transactions {
            cdebug!(COORDINATOR, "execute transaction {}, {}", tx.tx_type(), tx.hash());
            match services.tx_owner.get(tx.tx_type()) {
                Some(owner) => {
                    storage.create_checkpoint();
                    match owner.execute_transaction(session_id, tx) {
                        Ok(outcome) => {
                            outcomes.push(outcome);
                            storage.discard_checkpoint();
                            cdebug!(COORDINATOR, "execute transaction succeed {}, {}", tx.tx_type(), tx.hash());
                        }
                        Err(err) => {
                            storage.revert_to_the_checkpoint();
                            cdebug!(
                                COORDINATOR,
                                "execute transaction failed {}, {}, {:?}",
                                tx.tx_type(),
                                tx.hash(),
                                err
                            );
                        }
                    }
                }
                None => outcomes.push(TransactionOutcome::default()),
            }
        }

        Ok(outcomes)
    }

    fn prepare_block<'a>(
        &self,
        execution_id: ExecutionId,
        storage: &mut dyn StorageAccess,
        transactions: &mut dyn Iterator<Item = &'a TransactionWithMetadata>,
    ) -> Vec<(&'a Transaction, TransactionOutcome)> {
        cdebug!(COORDINATOR, "prepare block");
        let services = &self.services;

        let txs: Vec<_> = transactions.collect();
        let owned_txs: Vec<_> = txs.iter().map(|tx| (*tx).clone()).collect();
        let session_id = execution_id as SessionId;

        let SortedTxs {
            sorted,
            ..
        } = services.tx_sorter.sort_txs(session_id, &owned_txs);

        let mut tx_n_outcomes: Vec<(&'a Transaction, TransactionOutcome)> = Vec::new();
        let mut remaining_block_space = storage.max_body_size();
        let mut succeed_count = 0_u32;
        let mut fail_count = 0_u32;

        for index in sorted {
            let tx = &txs[index].tx;
            if let Some(owner) = services.tx_owner.get(tx.tx_type()) {
                cdebug!(COORDINATOR, "prepare_block: execute transaction {}, {}", tx.tx_type(), tx.hash());
                if remaining_block_space <= tx.size() as u64 {
                    break
                }
                storage.create_checkpoint();
                match owner.execute_transaction(session_id, &tx) {
                    Ok(outcome) => {
                        cdebug!(
                            COORDINATOR,
                            "prepare_block: execute transaction {}, {} succeed",
                            tx.tx_type(),
                            tx.hash()
                        );
                        storage.discard_checkpoint();
                        tx_n_outcomes.push((tx, outcome));
                        remaining_block_space -= tx.size() as u64;
                        succeed_count += 1;
                        continue
                    }
                    Err(err) => {
                        cdebug!(
                            COORDINATOR,
                            "prepare_block: execute transaction {}, {} error {:?}",
                            tx.tx_type(),
                            tx.hash(),
                            err
                        );
                        fail_count += 1;
                    }
                }
                storage.revert_to_the_checkpoint();
            } else {
                cdebug!(COORDINATOR, "prepare_block: can't find transaction owner of {}, {}", tx.tx_type(), tx.hash());
                fail_count += 1;
            }
        }
        cdebug!(COORDINATOR, "prepare_block: succeed: {} failed: {}", succeed_count, fail_count);
        tx_n_outcomes
    }

    fn close_block(&self, execution_id: ExecutionId) -> Result<BlockOutcome, CloseBlockError> {
        cdebug!(COORDINATOR, "close block");
        let services = &self.services;

        let session_id = execution_id as SessionId;
        let mut events = Vec::new();
        for owner in services.tx_owner.values() {
            events.extend(owner.block_closed(session_id)?.into_iter());
        }
        let (updated_validator_set, updated_chain_params) = services.update_consensus.update_consensus(session_id);

        self.end_session(session_id);

        Ok(BlockOutcome {
            updated_validator_set,
            updated_chain_params,
            events,
        })
    }
}

impl TxFilter for Coordinator {
    fn check_transaction(&self, tx: &Transaction) -> Result<(), ErrorCode> {
        let services = &self.services;

        match services.tx_owner.get(tx.tx_type()) {
            Some(owner) => owner.check_transaction(tx),
            // FIXME: proper error code management is required
            None => Err(ErrorCode::MAX),
        }
    }

    fn filter_transactions<'a>(
        &self,
        storage: &mut dyn StorageAccess,
        transactions: &mut dyn Iterator<Item = &'a TransactionWithMetadata>,
        memory_limit: Option<usize>,
        size_limit: Option<usize>,
    ) -> FilteredTxs<'a> {
        let services = &self.services;

        let txs: Vec<_> = transactions.collect();
        let owned_txs: Vec<_> = txs.iter().map(|tx| (*tx).clone()).collect();

        let session_id = self.new_session(storage);

        let SortedTxs {
            sorted,
            invalid,
        } = services.tx_sorter.sort_txs(session_id, &owned_txs);

        let memory_limit = memory_limit.unwrap_or(usize::MAX);
        let mut memory_usage = 0;
        let size_limit = size_limit.unwrap_or_else(|| txs.len());

        let low_priority = sorted
            .into_iter()
            .map(|i| &txs[i].tx)
            .enumerate()
            .skip_while(|(i, tx)| {
                memory_usage += (*tx).size();
                *i >= size_limit || memory_limit >= memory_usage
            })
            .map(|(_, tx)| tx)
            .collect();

        let invalid = invalid.into_iter().map(|i| &txs[i].tx).collect();
        self.end_session(session_id);

        FilteredTxs {
            invalid,
            low_priority,
        }
    }
}

impl GraphQlHandlerProvider for Coordinator {
    fn get(&self) -> Vec<(String, Arc<dyn HandleGraphQlRequest>)> {
        self.services.handle_graphqls.to_vec()
    }

    fn new_session_for_query(&self, storage: &mut dyn StorageAccess) -> crate::module::SessionId {
        self.new_session(storage)
    }

    fn end_session_for_query(&self, session: crate::module::SessionId) {
        self.end_session(session)
    }
}
