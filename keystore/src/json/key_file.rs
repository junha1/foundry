// Copyright 2015-2017 Parity Technologies (UK) Ltd.
// Copyright 2020 Kodebox, Inc.
// This file is part of Parity.

// Parity is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.

// Parity is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.

// You should have received a copy of the GNU General Public License
// along with Parity.  If not, see <http://www.gnu.org/licenses/>.

use super::{Crypto, Uuid, Version, H256};
use serde::{Serialize, Serializer};
use std::io::{Read, Write};

/// Public opaque type representing serializable `KeyFile`.
#[derive(Debug, PartialEq)]
pub struct OpaqueKeyFile {
    key_file: KeyFile,
}

impl Serialize for OpaqueKeyFile {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer, {
        self.key_file.serialize(serializer)
    }
}

impl<T> From<T> for OpaqueKeyFile
where
    T: Into<KeyFile>,
{
    fn from(val: T) -> Self {
        OpaqueKeyFile {
            key_file: val.into(),
        }
    }
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub struct KeyFile {
    pub id: Uuid,
    pub version: Version,
    pub crypto: Crypto,
    pub pubkey: Option<H256>,
    pub meta: Option<String>,
}

impl KeyFile {
    pub fn load<R>(reader: R) -> Result<Self, serde_json::Error>
    where
        R: Read, {
        serde_json::from_reader(reader)
    }

    pub fn write<W>(&self, writer: &mut W) -> Result<(), serde_json::Error>
    where
        W: Write, {
        serde_json::to_writer(writer, self)
    }
}

#[cfg(test)]
mod tests {
    use crate::json::{Aes128Ctr, Cipher, Crypto, Kdf, KeyFile, Scrypt, Uuid, Version};
    use std::str::FromStr;

    #[test]
    fn basic_keyfile() {
        let json = r#"
		{
			"pubkey": "0a6902c51384a15d1062cac3a4e62c8d0c2eb02b4de7fa0a304ce4f88ea482d0",
			"crypto": {
				"cipher": "aes-128-ctr",
				"ciphertext": "7203da0676d141b138cd7f8e1a4365f59cc1aa6978dc5443f364ca943d7cb4bc",
				"cipherparams": {
					"iv": "b5a7ec855ec9e2c405371356855fec83"
				},
				"kdf": "scrypt",
				"kdfparams": {
					"dklen": 32,
					"n": 262144,
					"p": 1,
					"r": 8,
					"salt": "1e8642fdf1f87172492c1412fc62f8db75d796cdfa9c53c3f2b11e44a2a1b209"
				},
				"mac": "46325c5d4e8c991ad2683d525c7854da387138b6ca45068985aa4959fa2b8c8f"
			},
			"id": "8777d9f6-7860-4b9b-88b7-0b57ee6b3a73",
			"version": 3,
			"meta": "{}"
		}"#;

        let expected = KeyFile {
            id: Uuid::from_str("8777d9f6-7860-4b9b-88b7-0b57ee6b3a73").unwrap(),
            version: Version::V3,
            pubkey: Some("0a6902c51384a15d1062cac3a4e62c8d0c2eb02b4de7fa0a304ce4f88ea482d0".into()),
            crypto: Crypto {
                cipher: Cipher::Aes128Ctr(Aes128Ctr {
                    iv: "b5a7ec855ec9e2c405371356855fec83".into(),
                }),
                ciphertext: "7203da0676d141b138cd7f8e1a4365f59cc1aa6978dc5443f364ca943d7cb4bc".into(),
                kdf: Kdf::Scrypt(Scrypt {
                    n: 262_144,
                    dklen: 32,
                    p: 1,
                    r: 8,
                    salt: "1e8642fdf1f87172492c1412fc62f8db75d796cdfa9c53c3f2b11e44a2a1b209".into(),
                }),
                mac: "46325c5d4e8c991ad2683d525c7854da387138b6ca45068985aa4959fa2b8c8f".into(),
            },
            meta: Some("{}".to_string()),
        };

        let keyfile: KeyFile = serde_json::from_str(json).unwrap();
        assert_eq!(keyfile, expected);
    }

    #[test]
    fn ignore_name_field() {
        let json = r#"
		{
			"pubkey": "0a6902c51384a15d1062cac3a4e62c8d0c2eb02b4de7fa0a304ce4f88ea482d0",
			"crypto": {
				"cipher": "aes-128-ctr",
				"ciphertext": "7203da0676d141b138cd7f8e1a4365f59cc1aa6978dc5443f364ca943d7cb4bc",
				"cipherparams": {
					"iv": "b5a7ec855ec9e2c405371356855fec83"
				},
				"kdf": "scrypt",
				"kdfparams": {
					"dklen": 32,
					"n": 262144,
					"p": 1,
					"r": 8,
					"salt": "1e8642fdf1f87172492c1412fc62f8db75d796cdfa9c53c3f2b11e44a2a1b209"
				},
				"mac": "46325c5d4e8c991ad2683d525c7854da387138b6ca45068985aa4959fa2b8c8f"
			},
			"id": "8777d9f6-7860-4b9b-88b7-0b57ee6b3a73",
			"version": 3,
			"name": "Test",
			"meta": "{}"
		}"#;

        let expected = KeyFile {
            id: Uuid::from_str("8777d9f6-7860-4b9b-88b7-0b57ee6b3a73").unwrap(),
            version: Version::V3,
            pubkey: Some("0a6902c51384a15d1062cac3a4e62c8d0c2eb02b4de7fa0a304ce4f88ea482d0".into()),
            crypto: Crypto {
                cipher: Cipher::Aes128Ctr(Aes128Ctr {
                    iv: "b5a7ec855ec9e2c405371356855fec83".into(),
                }),
                ciphertext: "7203da0676d141b138cd7f8e1a4365f59cc1aa6978dc5443f364ca943d7cb4bc".into(),
                kdf: Kdf::Scrypt(Scrypt {
                    n: 262_144,
                    dklen: 32,
                    p: 1,
                    r: 8,
                    salt: "1e8642fdf1f87172492c1412fc62f8db75d796cdfa9c53c3f2b11e44a2a1b209".into(),
                }),
                mac: "46325c5d4e8c991ad2683d525c7854da387138b6ca45068985aa4959fa2b8c8f".into(),
            },
            meta: Some("{}".to_string()),
        };

        let keyfile: KeyFile = serde_json::from_str(json).unwrap();
        assert_eq!(keyfile, expected);
    }

    #[test]
    fn address_is_an_optional_field() {
        const JSON: &str = r#"
        {
            "crypto": {
                "cipher": "aes-128-ctr",
                "ciphertext": "7203da0676d141b138cd7f8e1a4365f59cc1aa6978dc5443f364ca943d7cb4bc",
                "cipherparams": {
                    "iv": "b5a7ec855ec9e2c405371356855fec83"
                },
                "kdf": "scrypt",
                "kdfparams": {
                    "dklen": 32,
                    "n": 262144,
                    "p": 1,
                    "r": 8,
                    "salt": "1e8642fdf1f87172492c1412fc62f8db75d796cdfa9c53c3f2b11e44a2a1b209"
                },
                "mac": "46325c5d4e8c991ad2683d525c7854da387138b6ca45068985aa4959fa2b8c8f"
            },
            "id": "8777d9f6-7860-4b9b-88b7-0b57ee6b3a73",
            "version": 3,
            "meta": "{}"
        }"#;

        let expected = KeyFile {
            id: Uuid::from_str("8777d9f6-7860-4b9b-88b7-0b57ee6b3a73").unwrap(),
            version: Version::V3,
            pubkey: None,
            crypto: Crypto {
                cipher: Cipher::Aes128Ctr(Aes128Ctr {
                    iv: "b5a7ec855ec9e2c405371356855fec83".into(),
                }),
                ciphertext: "7203da0676d141b138cd7f8e1a4365f59cc1aa6978dc5443f364ca943d7cb4bc".into(),
                kdf: Kdf::Scrypt(Scrypt {
                    n: 262_144,
                    dklen: 32,
                    p: 1,
                    r: 8,
                    salt: "1e8642fdf1f87172492c1412fc62f8db75d796cdfa9c53c3f2b11e44a2a1b209".into(),
                }),
                mac: "46325c5d4e8c991ad2683d525c7854da387138b6ca45068985aa4959fa2b8c8f".into(),
            },
            meta: Some("{}".to_string()),
        };

        let keyfile: KeyFile = serde_json::from_str(JSON).unwrap();
        assert_eq!(keyfile, expected);
    }

    #[test]
    fn capital_crypto_is_not_allowed() {
        const JSON: &str = r#"
		{
			"pubkey": "0a6902c51384a15d1062cac3a4e62c8d0c2eb02b4de7fa0a304ce4f88ea482d0",
			"Crypto": {
				"cipher": "aes-128-ctr",
				"ciphertext": "7203da0676d141b138cd7f8e1a4365f59cc1aa6978dc5443f364ca943d7cb4bc",
				"cipherparams": {
					"iv": "b5a7ec855ec9e2c405371356855fec83"
				},
				"kdf": "scrypt",
				"kdfparams": {
					"dklen": 32,
					"n": 262144,
					"p": 1,
					"r": 8,
					"salt": "1e8642fdf1f87172492c1412fc62f8db75d796cdfa9c53c3f2b11e44a2a1b209"
				},
				"mac": "46325c5d4e8c991ad2683d525c7854da387138b6ca45068985aa4959fa2b8c8f"
			},
			"id": "8777d9f6-7860-4b9b-88b7-0b57ee6b3a73",
			"version": 3
		}"#;

        let must_fail = ::std::panic::catch_unwind(|| {
            serde_json::from_str::<KeyFile>(JSON).unwrap();
        });
        assert!(must_fail.is_err());
    }

    #[test]
    fn to_and_from_json() {
        let file = KeyFile {
            id: "8777d9f6-7860-4b9b-88b7-0b57ee6b3a73".into(),
            version: Version::V3,
            pubkey: Some("0a6902c51384a15d1062cac3a4e62c8d0c2eb02b4de7fa0a304ce4f88ea482d0".into()),
            crypto: Crypto {
                cipher: Cipher::Aes128Ctr(Aes128Ctr {
                    iv: "b5a7ec855ec9e2c405371356855fec83".into(),
                }),
                ciphertext: "7203da0676d141b138cd7f8e1a4365f59cc1aa6978dc5443f364ca943d7cb4bc".into(),
                kdf: Kdf::Scrypt(Scrypt {
                    n: 262_144,
                    dklen: 32,
                    p: 1,
                    r: 8,
                    salt: "1e8642fdf1f87172492c1412fc62f8db75d796cdfa9c53c3f2b11e44a2a1b209".into(),
                }),
                mac: "46325c5d4e8c991ad2683d525c7854da387138b6ca45068985aa4959fa2b8c8f".into(),
            },
            meta: None,
        };

        let serialized = serde_json::to_string(&file).unwrap();
        println!("{}", serialized);
        let deserialized = serde_json::from_str(&serialized).unwrap();

        assert_eq!(file, deserialized);
    }
}
