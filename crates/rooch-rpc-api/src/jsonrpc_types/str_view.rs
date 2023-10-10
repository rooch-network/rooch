// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use ethers::types::{H160, H256};
use move_core_types::account_address::AccountAddress;
use schemars::gen::SchemaGenerator;
use schemars::schema::{InstanceType, Schema, SchemaObject};
use schemars::JsonSchema;
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use std::str::FromStr;

/// StrVeiw is a wrapper around T that implements Serialize and Deserialize for jsonrpc
#[derive(Debug, PartialEq, Hash, Eq, Clone, Copy, PartialOrd, Ord)]
pub struct StrView<T>(pub T);

impl<T> JsonSchema for StrView<T> {
    fn schema_name() -> String {
        std::any::type_name::<T>().to_owned()
    }

    fn json_schema(_gen: &mut SchemaGenerator) -> Schema {
        SchemaObject {
            instance_type: Some(InstanceType::String.into()),
            ..Default::default()
        }
        .into()
    }
}

impl<T> From<T> for StrView<T> {
    fn from(t: T) -> Self {
        Self(t)
    }
}

impl<T> Serialize for StrView<T>
where
    Self: ToString,
{
    fn serialize<S>(&self, serializer: S) -> Result<<S as Serializer>::Ok, <S as Serializer>::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(&self.to_string())
    }
}

impl<'de, T> Deserialize<'de> for StrView<T>
where
    Self: FromStr,
    <Self as FromStr>::Err: std::fmt::Display,
{
    fn deserialize<D>(deserializer: D) -> Result<Self, <D as Deserializer<'de>>::Error>
    where
        D: Deserializer<'de>,
    {
        let s = <String>::deserialize(deserializer)?;

        StrView::<T>::from_str(&s).map_err(serde::de::Error::custom)
    }
}

impl<T> Default for StrView<T>
where
    T: Default,
{
    fn default() -> Self {
        Self(T::default())
    }
}

macro_rules! impl_str_view_for {
    ($($t:ty)*) => {$(
    impl std::fmt::Display for StrView<$t> {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            write!(f, "{}", self.0)
        }
    }
    impl std::str::FromStr for StrView<$t> {
        type Err = <$t as std::str::FromStr>::Err;
        fn from_str(s: &str) -> Result<Self, Self::Err> {
            <$t>::from_str(s).map(StrView)
        }
    }
    impl From<StrView<$t>> for $t {
        fn from(view: StrView<$t>) -> $t {
            view.0
        }
    }
    )*}
}

impl_str_view_for! {u64 i64 u128 i128 u16 i16 u32 i32 move_core_types::u256::U256 H160 H256}

impl std::fmt::Display for StrView<Vec<u8>> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "0x{}", hex::encode(&self.0))
    }
}

impl FromStr for StrView<Vec<u8>> {
    type Err = anyhow::Error;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(Self(hex::decode(s.strip_prefix("0x").unwrap_or(s))?))
    }
}

impl From<StrView<Vec<u8>>> for Vec<u8> {
    fn from(value: StrView<Vec<u8>>) -> Self {
        value.0
    }
}

impl AsRef<[u8]> for StrView<Vec<u8>> {
    fn as_ref(&self) -> &[u8] {
        self.0.as_ref()
    }
}

impl std::fmt::Display for StrView<AccountAddress> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        //Ensure append `0x` before the address, and output full address
        //The Display implemention of AccountAddress has not `0x` prefix
        write!(f, "0x{}", self.0)
    }
}

impl FromStr for StrView<AccountAddress> {
    type Err = anyhow::Error;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        // AccountAddress::from_str suppport both 0xABCD and ABCD
        Ok(StrView(AccountAddress::from_str(s)?))
    }
}

impl From<StrView<AccountAddress>> for AccountAddress {
    fn from(value: StrView<AccountAddress>) -> Self {
        value.0
    }
}

impl FromStr for StrView<ethers::types::U256> {
    type Err = anyhow::Error;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(StrView(ethers::types::U256::from_str(s)?))
    }
}

impl From<StrView<ethers::types::U256>> for ethers::types::U256 {
    fn from(value: StrView<ethers::types::U256>) -> Self {
        value.0
    }
}

impl std::fmt::Display for StrView<ethers::types::U256> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        //The U256 in ethereum should display as hex string with `0x` prefix
        write!(f, "0x{:x}", self.0)
    }
}

impl FromStr for StrView<ethers::types::U64> {
    type Err = anyhow::Error;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(StrView(ethers::types::U64::from_str(s)?))
    }
}

impl From<StrView<ethers::types::U64>> for ethers::types::U64 {
    fn from(value: StrView<ethers::types::U64>) -> Self {
        value.0
    }
}

impl std::fmt::Display for StrView<ethers::types::U64> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        //The U64 in ethereum should display as hex string with `0x` prefix
        write!(f, "0x{:x}", self.0)
    }
}
