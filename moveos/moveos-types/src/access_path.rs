// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use crate::moveos_std::account::Account;
use crate::moveos_std::move_module::ModuleStore;
use crate::state::KeyState;
use crate::{
    move_types::{random_identity, random_struct_tag},
    moveos_std::object::ObjectID,
};
use anyhow::{bail, ensure, Result};
use move_core_types::language_storage::ModuleId;
use move_core_types::{
    account_address::AccountAddress, identifier::Identifier, language_storage::StructTag,
};
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use std::str::FromStr;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum Path {
    /// Get Object
    Object { object_ids: Vec<ObjectID> },
    /// Get account resources
    Resource {
        account: AccountAddress,
        resource_types: Vec<StructTag>,
    },
    /// Get account modules
    Module {
        account: AccountAddress,
        module_names: Vec<Identifier>,
    },
    Fields {
        object_id: ObjectID,
        fields: Vec<KeyState>,
    },
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum StateQuery {
    Objects(Vec<ObjectID>),
    Fields(ObjectID, Vec<KeyState>),
}

impl StateQuery {
    pub fn into_list_query(self) -> Result<ObjectID> {
        match self {
            StateQuery::Objects(object_ids) => {
                ensure!(
                    object_ids.is_empty(),
                    "List query does not support specific object id"
                );
                Ok(ObjectID::root())
            }
            StateQuery::Fields(object_id, fields) => {
                ensure!(fields.is_empty(), "List query does not support fields");
                Ok(object_id)
            }
        }
    }

    pub fn into_fields_query(self) -> Result<Vec<(ObjectID, KeyState)>> {
        match self {
            StateQuery::Objects(object_ids) => {
                ensure!(!object_ids.is_empty(), "Please specify object id");
                Ok(object_ids
                    .into_iter()
                    .map(|id| (id.parent().unwrap_or(ObjectID::root()), id.to_key()))
                    .collect())
            }
            StateQuery::Fields(object_id, fields) => {
                // ensure!(!fields.is_empty(), "Please specify fields");
                Ok(fields
                    .into_iter()
                    .map(|field| (object_id.clone(), field))
                    .collect())
            }
        }
    }
}

impl std::fmt::Display for Path {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Path::Object { object_ids } => {
                write!(
                    f,
                    "/object/{}",
                    object_ids
                        .iter()
                        .map(|object_id| object_id.to_string())
                        .collect::<Vec<_>>()
                        .join(",")
                )?;
            }
            Path::Resource {
                account,
                resource_types,
            } => {
                write!(
                    f,
                    "/resource/{}/{}",
                    account.to_hex_literal(),
                    resource_types
                        .iter()
                        .map(|struct_tag| struct_tag.to_string())
                        .collect::<Vec<_>>()
                        .join(",")
                )?;
            }
            Path::Module {
                account,
                module_names,
            } => {
                write!(
                    f,
                    "/module/{}/{}",
                    account.to_hex_literal(),
                    module_names
                        .iter()
                        .map(|module_name| module_name.to_string())
                        .collect::<Vec<_>>()
                        .join(",")
                )?;
            }
            Path::Fields { object_id, fields } => {
                write!(
                    f,
                    "/fields/{}/{}",
                    object_id,
                    fields
                        .iter()
                        .map(|key| key.to_string())
                        .collect::<Vec<_>>()
                        .join(",")
                )?;
            }
        }
        Ok(())
    }
}

impl FromStr for Path {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let s = s.trim_start_matches('/');
        let mut iter = s.split('/');
        let path_type = iter
            .next()
            .ok_or_else(|| anyhow::anyhow!("Invalid access path"))?;
        match path_type {
            "object" => {
                let object_ids = iter.next().unwrap_or("");
                let object_ids = object_ids
                    .split(',')
                    .map(ObjectID::from_str)
                    .collect::<Result<Vec<_>, _>>()?;
                Ok(Path::Object { object_ids })
            }
            "resource" => {
                let account = iter
                    .next()
                    .ok_or_else(|| anyhow::anyhow!("Invalid access path"))?;
                let account = AccountAddress::from_hex_literal(account)?;
                let resource_types = match iter.next() {
                    Some(v) => v
                        .split(',')
                        .map(StructTag::from_str)
                        .collect::<Result<Vec<_>, _>>()?,
                    None => vec![],
                };

                Ok(Path::Resource {
                    account,
                    resource_types,
                })
            }
            "module" => {
                let account = iter
                    .next()
                    .ok_or_else(|| anyhow::anyhow!("Invalid access path"))?;
                let account = AccountAddress::from_hex_literal(account)?;
                let module_names = match iter.next() {
                    Some(v) => v
                        .split(',')
                        .map(Identifier::from_str)
                        .collect::<Result<Vec<_>, _>>()?,

                    None => {
                        bail!("Invalid access path, require module name")
                    }
                };

                Ok(Path::Module {
                    account,
                    module_names,
                })
            }
            "table" | "fields" => {
                let object_id_str = iter.next().ok_or_else(|| {
                    anyhow::anyhow!("Invalid access path, require $object_id in /fields/$object_id")
                })?;
                let object_id = ObjectID::from_str(object_id_str)?;

                let fields = match iter.next() {
                    Some(v) => {
                        if v.trim().is_empty() {
                            vec![]
                        } else {
                            v.split(',')
                                .map(|key| {
                                    KeyState::from_str(key).map_err(|e| {
                                        anyhow::anyhow!(
                                            "Invalid access path key: {}, err: {:?}",
                                            key,
                                            e
                                        )
                                    })
                                })
                                .collect::<Result<Vec<_>, _>>()?
                        }
                    }
                    None => vec![],
                };

                Ok(Path::Fields { object_id, fields })
            }
            _ => Err(anyhow::anyhow!("Invalid access path: {}", s)),
        }
    }
}

/// StateDB query path
///
/// 1. /object/$object_id1[,$object_id2]*
/// 2. /resource/$account_address/$resource_type1[,$resource_type2]*
/// 3. /module/$account_address/$module_name1[,$module_name2]*
/// 4. /fields/$object_id/$field_key_state1[,$field_key_state2]*
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct AccessPath(pub Path);

impl AccessPath {
    pub fn object(object_id: ObjectID) -> Self {
        AccessPath(Path::Object {
            object_ids: vec![object_id],
        })
    }

    pub fn objects(object_ids: Vec<ObjectID>) -> Self {
        AccessPath(Path::Object { object_ids })
    }

    pub fn resource(account: AccountAddress, resource_type: StructTag) -> Self {
        AccessPath(Path::Resource {
            account,
            resource_types: vec![resource_type],
        })
    }

    pub fn resources(account: AccountAddress, resource_types: Vec<StructTag>) -> Self {
        AccessPath(Path::Resource {
            account,
            resource_types,
        })
    }

    pub fn module(account: AccountAddress, module_name: Identifier) -> Self {
        AccessPath(Path::Module {
            account,
            module_names: vec![module_name],
        })
    }

    pub fn modules(account: AccountAddress, module_names: Vec<Identifier>) -> Self {
        AccessPath(Path::Module {
            account,
            module_names,
        })
    }

    pub fn fields(object_id: ObjectID, fields: Vec<KeyState>) -> Self {
        AccessPath(Path::Fields { object_id, fields })
    }

    pub fn fields_without_keys(object_id: ObjectID) -> Self {
        AccessPath(Path::Fields {
            object_id,
            fields: vec![],
        })
    }

    /// Convert AccessPath to StateQuery, return the ObjectID and field keys
    pub fn into_state_query(self) -> StateQuery {
        match self.0 {
            Path::Fields { object_id, fields } => StateQuery::Fields(object_id, fields),
            Path::Object { object_ids } => {
                if object_ids.is_empty() {
                    StateQuery::Fields(ObjectID::root(), vec![])
                } else {
                    StateQuery::Objects(object_ids)
                }
            }
            Path::Module {
                account,
                module_names,
            } => {
                let module_object_id = ModuleStore::module_store_id();
                let keys = module_names
                    .into_iter()
                    .map(|name| {
                        let module_id = ModuleId::new(account, name);
                        KeyState::from_module_id(&module_id)
                    })
                    .collect();
                StateQuery::Fields(module_object_id, keys)
            }
            Path::Resource {
                account,
                resource_types,
            } => {
                let account_object_id = Account::account_object_id(account);
                let keys = resource_types
                    .into_iter()
                    .map(|tag| KeyState::from_struct_tag(&tag))
                    .collect();
                StateQuery::Fields(account_object_id, keys)
            }
        }
    }

    pub fn is_object(&self) -> bool {
        matches!(self.0, Path::Object { .. })
    }

    pub fn is_resource(&self) -> bool {
        matches!(self.0, Path::Resource { .. })
    }

    pub fn random_module() -> AccessPath {
        Self::random_module_with_fixed_address(AccountAddress::random())
    }

    pub fn random_module_with_fixed_address(addr: AccountAddress) -> AccessPath {
        Self::module(addr, random_identity())
    }

    pub fn random_resource() -> AccessPath {
        Self::random_resource_with_fixed_address(AccountAddress::random())
    }

    pub fn random_resource_with_fixed_address(addr: AccountAddress) -> AccessPath {
        Self::resource(addr, random_struct_tag())
    }
}

impl std::fmt::Display for AccessPath {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)?;
        Ok(())
    }
}

impl FromStr for AccessPath {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let path = s.parse::<Path>()?;
        Ok(AccessPath(path))
    }
}

// AccessPath always serialize and deserilaize as string
impl Serialize for AccessPath {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(self.to_string().as_str())
    }
}

impl<'de> Deserialize<'de> for AccessPath {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        s.parse::<AccessPath>().map_err(serde::de::Error::custom)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::move_std::string::MoveString;
    use crate::state::MoveType;

    fn test_path_roundtrip(path: &str) {
        let path = path.parse::<Path>().unwrap();
        let path_str = path.to_string();
        let path2 = path_str.parse::<Path>().unwrap();
        assert_eq!(path, path2);
    }

    #[test]
    pub fn test_table_path() -> Result<()> {
        let key1 = KeyState::new("0x12".as_bytes().to_vec(), MoveString::type_tag());
        let key2 = KeyState::new("0x13".as_bytes().to_vec(), MoveString::type_tag());
        let key3 = KeyState::new("key1".as_bytes().to_vec(), MoveString::type_tag());
        let key4 = KeyState::new("key2".as_bytes().to_vec(), MoveString::type_tag());
        println!("test_table_path key1 {}", key1.to_string());
        println!("test_table_path key2 {}", key2.to_string());
        println!("test_table_path key3 {}", key3.to_string());
        println!("test_table_path key4 {}", key4.to_string());

        Ok(())
    }

    #[test]
    pub fn test_path() {
        test_path_roundtrip("/object/0x1");
        test_path_roundtrip("/object/0x1,0x2");
        test_path_roundtrip("/resource/0x1/0x2::m::S");
        test_path_roundtrip("/resource/0x1/0x2::m1::S1,0x3::m2::S2");
        test_path_roundtrip("/module/0x2/m1");
        test_path_roundtrip("/module/0x2/m1,m2");
        // test_path_roundtrip("/table/0x1/0x12");
        // test_path_roundtrip("/table/0x1/0x12,0x13");
        // test_path_roundtrip("/table/0x1/key1,key2");
        test_path_roundtrip("/table/0x1/0x043078313207000000000000000000000000000000000000000000000000000000000000000106737472696e6706537472696e6700");
        test_path_roundtrip("/table/0x1/0x043078313207000000000000000000000000000000000000000000000000000000000000000106737472696e6706537472696e6700,0x043078313307000000000000000000000000000000000000000000000000000000000000000106737472696e6706537472696e6700");
        test_path_roundtrip("/table/0x1/0x046b65793107000000000000000000000000000000000000000000000000000000000000000106737472696e6706537472696e6700,0x046b65793207000000000000000000000000000000000000000000000000000000000000000106737472696e6706537472696e6700");
    }
}
