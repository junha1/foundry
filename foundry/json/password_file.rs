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

use super::password_entry::PasswordEntry;
use serde::{Deserialize, Serialize};
use std::io::Read;

#[derive(Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct PasswordFile(Vec<PasswordEntry>);

impl PasswordFile {
    pub fn load<R>(reader: R) -> Result<Self, serde_json::Error>
    where
        R: Read, {
        serde_json::from_reader(reader)
    }

    pub fn entries(&self) -> &[PasswordEntry] {
        self.0.as_slice()
    }
}

#[cfg(test)]
mod tests {
    use super::super::password_entry::PasswordEntry;
    use super::PasswordFile;

    #[test]
    fn password_file() {
        let json = r#"
		[
            {
                "address": "4cnj73b1X2_K3XI_PJSXAQHZG0BD-VO4-TnhS3WEOy8Ym7UmT1Utc0",
                "password": "mypass1"
            },
            {
                "address": "fys3db1kOrI_rXyaTx9U2_RP-SlNK1q0LRXxYeQGBI1av35drZQtc0",
                "password": "mypass2"
            }
		]"#;

        let expected = PasswordFile(vec![
            PasswordEntry {
                address: "4cnj73b1X2_K3XI_PJSXAQHZG0BD-VO4-TnhS3WEOy8Ym7UmT1Utc0".into(),
                password: "mypass1".into(),
            },
            PasswordEntry {
                address: "fys3db1kOrI_rXyaTx9U2_RP-SlNK1q0LRXxYeQGBI1av35drZQtc0".into(),
                password: "mypass2".into(),
            },
        ]);

        let pf: PasswordFile = serde_json::from_str(json).unwrap();
        assert_eq!(pf, expected);
    }
}
