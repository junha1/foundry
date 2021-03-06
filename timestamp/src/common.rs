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
// GNU Affero General Public License for more details.
//
// You should have received a copy of the GNU General Public License
// along with this program.  If not, see <https://www.gnu.org/licenses/>.

pub mod state_machine;
mod state_manager;

use ckey::{verify, Ed25519Public as Public, Signature};
pub(crate) use foundry_graphql_types::*;
use serde::{Deserialize, Serialize};
pub use state_manager::StateManager;

pub type TxSeq = u64;

pub fn assert_empty_arg(arg: &[u8]) -> Result<(), String> {
    let a: std::collections::HashMap<String, String> = serde_cbor::from_slice(arg).map_err(|err| err.to_string())?;
    if a.is_empty() {
        Ok(())
    } else {
        Err(format!("This arg is not empty: {:?}", a))
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct SignedTransaction {
    pub signature: Signature,
    pub signer_public: Public,
    pub action: Vec<u8>,
}

impl SignedTransaction {
    pub fn verify(&self) -> Result<(), ()> {
        if verify(&self.signature, &self.action, &self.signer_public) {
            Ok(())
        } else {
            Err(())
        }
    }
}

pub fn handle_gql_query<T: async_graphql::ObjectType + Send + Sync + 'static>(
    runtime: &tokio::runtime::Handle,
    root: T,
    query: &str,
    variables: &str,
) -> String {
    let variables = if let Ok(s) = (|| -> Result<_, ()> {
        Ok(async_graphql::context::Variables::from_json(serde_json::from_str(variables).map_err(|_| ())?))
    })() {
        s
    } else {
        return "Failed to parse JSON".to_owned()
    };

    let schema = async_graphql::Schema::new(root, async_graphql::EmptyMutation, async_graphql::EmptySubscription);
    let request = async_graphql::Request::new(query).variables(variables);
    let res = schema.execute(request);
    serde_json::to_string(&runtime.block_on(res)).unwrap()
}
