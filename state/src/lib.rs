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

extern crate codechain_crypto as ccrypto;
extern crate codechain_db as cdb;
#[macro_use]
extern crate codechain_logger as clogger;
extern crate codechain_key as ckey;
extern crate codechain_types as ctypes;
#[macro_use]
extern crate log;
extern crate rlp_derive;

mod cache;
mod checkpoint;
mod db;
mod error;
mod impls;
mod item;
mod stake;
mod traits;

pub mod tests;

pub use crate::checkpoint::{CheckpointId, StateWithCheckpoint};
pub use crate::db::StateDB;
pub use crate::error::Error as StateError;
pub use crate::impls::{ModuleLevelState, TopLevelState};
pub use crate::item::action_data::ActionData;
pub use crate::item::metadata::{Metadata, MetadataAddress};
pub use crate::item::module::{Module, ModuleAddress};
pub use crate::item::module_datum::{ModuleDatum, ModuleDatumAddress};
pub use crate::item::stake::CurrentValidators;
pub use crate::item::validator_set::{CurrentValidatorSet, NextValidatorSet, SimpleValidator};
pub use crate::stake::StakeKeyBuilder;
pub use crate::traits::{StateWithCache, TopState, TopStateView};

use crate::cache::CacheableItem;

pub type StateResult<T> = Result<T, StateError>;
