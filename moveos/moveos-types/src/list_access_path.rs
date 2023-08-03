// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use crate::object::{NamedTableID, ObjectID};
use move_core_types::account_address::AccountAddress;
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use std::str::FromStr;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum PathList {
    /// Get account resources list
    Resource { account: AccountAddress },
    /// Get account modules list
    Module { account: AccountAddress },
    /// Get table values list by table handle
    Table { table_handle: ObjectID },
}

impl std::fmt::Display for PathList {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            PathList::Resource { account } => {
                write!(f, "/resource/{}", account.to_hex_literal(),)?;
            }
            PathList::Module { account } => {
                write!(f, "/module/{}", account.to_hex_literal(),)?;
            }
            PathList::Table { table_handle } => {
                write!(f, "/table/{}", table_handle,)?;
            }
        }
        Ok(())
    }
}

impl FromStr for PathList {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let s = s.trim_start_matches('/');
        let mut iter = s.split('/');
        let path_type = iter.next().ok_or_else(|| anyhow::anyhow!("Invalid path"))?;
        match path_type {
            "resource" => {
                let account = iter.next().ok_or_else(|| anyhow::anyhow!("Invalid path"))?;
                let account = AccountAddress::from_hex_literal(account)?;
                Ok(PathList::Resource { account })
            }
            "module" => {
                let account = iter.next().ok_or_else(|| anyhow::anyhow!("Invalid path"))?;
                let account = AccountAddress::from_hex_literal(account)?;
                Ok(PathList::Module { account })
            }
            "table" => {
                let table_handle = iter.next().ok_or_else(|| anyhow::anyhow!("Invalid path"))?;
                let table_handle = ObjectID::from_str(table_handle)?;
                Ok(PathList::Table { table_handle })
            }
            _ => Err(anyhow::anyhow!("Invalid path: {}", s)),
        }
    }
}

/// StateDB list query path
///
/// 1. /resource/$account_address
/// 2. /module/$account_address
/// 3. /table/$table_handle
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct AccessPathList(pub PathList);

impl AccessPathList {
    pub fn resource(account: AccountAddress) -> Self {
        AccessPathList(PathList::Resource { account })
    }

    pub fn resources(account: AccountAddress) -> Self {
        AccessPathList(PathList::Resource { account })
    }

    pub fn module(account: AccountAddress) -> Self {
        AccessPathList(PathList::Module { account })
    }

    pub fn modules(account: AccountAddress) -> Self {
        AccessPathList(PathList::Module { account })
    }

    pub fn table(table_handle: ObjectID) -> Self {
        AccessPathList(PathList::Table { table_handle })
    }

    /// Convert AccessPathList to TableQuery, return the table handle
    /// All other AccessPathList is a shortcut for TableQuery
    pub fn into_table_query(self) -> ObjectID {
        match self.0 {
            PathList::Table { table_handle } => table_handle,
            PathList::Module { account } => NamedTableID::Module(account).to_object_id(),
            PathList::Resource { account } => NamedTableID::Resource(account).to_object_id(),
        }
    }
}

impl std::fmt::Display for AccessPathList {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)?;
        Ok(())
    }
}

impl FromStr for AccessPathList {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let path = s.parse::<PathList>()?;
        Ok(AccessPathList(path))
    }
}

// AccessPathList always serialize and deserilaize as string
impl Serialize for AccessPathList {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(self.to_string().as_str())
    }
}

impl<'de> Deserialize<'de> for AccessPathList {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        s.parse::<AccessPathList>()
            .map_err(serde::de::Error::custom)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_path_roundtrip(path: &str) {
        let path = path.parse::<PathList>().unwrap();
        let path_str = path.to_string();
        let path2 = path_str.parse::<PathList>().unwrap();
        assert_eq!(path, path2);
    }

    #[test]
    pub fn test_path() {
        test_path_roundtrip("/resource/0x1");
        test_path_roundtrip("/module/0x2");
        test_path_roundtrip("/table/0x1");
    }
}
