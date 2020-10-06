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

// You should run `cargo test` with `--test-threads=1`, if you use `demo` feature.
#![cfg(feature = "demo")]

use std::thread::sleep;
use std::time::Duration;

#[test]
fn run() {
    let mut child = foundry_demo::run_node(4444);
    sleep(Duration::from_secs(3));
    child.kill().unwrap();
    child.wait().unwrap();
}

#[actix_rt::test]
async fn ping() {
    let mut child = foundry_demo::run_node(5555);
    sleep(Duration::from_secs(3));
    let x = foundry_demo::request_query(5555, "ping", "aaaa", "aaaa").await;
    assert_eq!(x, "Module not found: ping");
    child.kill().unwrap();
    child.wait().unwrap();
}
