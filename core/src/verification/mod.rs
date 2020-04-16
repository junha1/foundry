// Copyright 2018, 2020 Kodebox, Inc.
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

mod canon_verifier;
mod noop_verifier;
pub mod queue;
#[cfg_attr(feature = "cargo-clippy", allow(clippy::module_inception))]
mod verification;
mod verifier;

pub use self::canon_verifier::CanonVerifier;
pub use self::noop_verifier::NoopVerifier;
pub use self::queue::{BlockQueue, Config as QueueConfig};
pub use self::verification::*;
pub use self::verifier::Verifier;
