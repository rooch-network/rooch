// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use move_core_types::u256;
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use std::str::FromStr;

/// StrVeiw is a wrapper around T that implements Serialize and Deserialize for jsonrpc
#[derive(Debug, PartialEq, Hash, Eq, Clone, Copy, PartialOrd, Ord)]
pub struct StrView<T>(pub T);

//TODO define JsonSchema
// impl<T> JsonSchema for StrView<T> {
//     fn schema_name() -> String {
//         std::any::type_name::<T>().to_owned()
//     }

//     fn json_schema(_gen: &mut SchemaGenerator) -> Schema {
//         SchemaObject {
//             instance_type: Some(InstanceType::String.into()),
//             ..Default::default()
//         }
//         .into()
//     }
// }

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

impl_str_view_for! {u64 i64 u128 i128 u16 i16 u32 i32 u256::U256}

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
