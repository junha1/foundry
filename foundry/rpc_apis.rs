// Copyright 2018-2019 Kodebox, Inc.
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

use crate::config::Config;
use ccore::{AccountProvider, Client, Miner};
use clogger::SLOGGER;
use cnetwork::{EventSender, NetworkControl};
use crpc::{MetaIoHandler, Middleware, Params, Value};
use csync::BlockSyncEvent;
use std::sync::Arc;

pub struct ApiDependencies {
    pub client: Arc<Client>,
    pub miner: Arc<Miner>,
    pub network_control: Arc<dyn NetworkControl>,
    pub account_provider: Arc<AccountProvider>,
    pub block_sync: Option<EventSender<BlockSyncEvent>>,
}

impl ApiDependencies {
    pub fn extend_api(&self, config: &Config, handler: &mut MetaIoHandler<(), impl Middleware<()>>) {
        use crpc::v1::*;
        handler.extend_with(ChainClient::new(Arc::clone(&self.client)).to_delegate());
        handler.extend_with(MempoolClient::new(Arc::clone(&self.client)).to_delegate());
        handler.extend_with(SnapshotClient::new(Arc::clone(&self.client), config.snapshot.path.clone()).to_delegate());
        if config.rpc.enable_devel_api {
            handler.extend_with(
                DevelClient::new(Arc::clone(&self.client), Arc::clone(&self.miner), self.block_sync.clone())
                    .to_delegate(),
            );
        }
        handler.extend_with(NetClient::new(Arc::clone(&self.network_control)).to_delegate());
    }
}

pub fn setup_rpc<M: Middleware<()>>(mut handler: MetaIoHandler<(), M>) -> MetaIoHandler<(), M> {
    handler.add_method("ping", |_params: Params| Ok(Value::String("pong".to_string())));
    handler.add_method("version", |_params: Params| Ok(Value::String(env!("CARGO_PKG_VERSION").to_string())));
    handler.add_method("commitHash", |_params: Params| Ok(Value::String(env!("VERGEN_SHA").to_string())));

    handler.add_method("slog", |_params: Params| {
        let logs = SLOGGER.get_logs();
        Ok(Value::Array(logs))
    });
    handler
}
