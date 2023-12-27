// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use moveos_types::move_std::string::MoveString;
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

// Because the max value of json number is less than u64::MAX, so we need to use string to represent usize, u64, i64, u128, i128, U256
impl_str_view_for! {usize u64 i64 u128 i128 move_core_types::u256::U256}

pub type BytesView = StrView<Vec<u8>>;

impl std::fmt::Display for BytesView {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "0x{}", hex::encode(&self.0))
    }
}

impl FromStr for BytesView {
    type Err = anyhow::Error;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(Self(hex::decode(s.strip_prefix("0x").unwrap_or(s))?))
    }
}

impl From<BytesView> for Vec<u8> {
    fn from(value: BytesView) -> Self {
        value.0
    }
}

impl AsRef<[u8]> for BytesView {
    fn as_ref(&self) -> &[u8] {
        self.0.as_ref()
    }
}

/// We do not define U256View, because StrView<ethers::types::U256> is different from StrView<move_core_types::u256::U256>
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
        write!(f, "{:#x}", self.0)
    }
}

/// StrView<ethers::types::U64> is different from StrView<u64>
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
        write!(f, "{:#x}", self.0)
    }
}

pub type H64View = StrView<ethers::types::H64>;

impl FromStr for H64View {
    type Err = anyhow::Error;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(StrView(ethers::types::H64::from_str(s)?))
    }
}

impl From<H64View> for ethers::types::H64 {
    fn from(value: H64View) -> Self {
        value.0
    }
}

impl std::fmt::Display for H64View {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        //The H64 should display fully hex string with `0x` prefix
        write!(f, "{:#x}", self.0)
    }
}

pub type H160View = StrView<ethers::types::H160>;

impl FromStr for H160View {
    type Err = anyhow::Error;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(StrView(ethers::types::H160::from_str(s)?))
    }
}

impl From<H160View> for ethers::types::H160 {
    fn from(value: H160View) -> Self {
        value.0
    }
}

impl std::fmt::Display for H160View {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        //The H160 should display fully hex string with `0x` prefix
        write!(f, "{:#x}", self.0)
    }
}

pub type H256View = StrView<ethers::types::H256>;

impl FromStr for H256View {
    type Err = anyhow::Error;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(StrView(ethers::types::H256::from_str(s)?))
    }
}

impl From<H256View> for ethers::types::H256 {
    fn from(value: H256View) -> Self {
        value.0
    }
}

impl std::fmt::Display for H256View {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        //The H256 should display fully hex string with `0x` prefix
        write!(f, "{:#x}", self.0)
    }
}

//
// // pub type H256View = StrView<ethers::types::H256>;
pub type MoveStringView = StrView<MoveString>;

impl FromStr for MoveStringView {
    type Err = anyhow::Error;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(StrView(MoveString::from_str(s)?))
        // Ok(Self(MoveString::from_str(
        //     s.strip_prefix("0x").unwrap_or(s),
        // )?))
    }
}

impl From<MoveStringView> for MoveString {
    fn from(value: MoveStringView) -> Self {
        value.0
    }
}

impl std::fmt::Display for MoveStringView {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
        // write!(f, "0x{}", hex::encode(&self.0))
    }
}
