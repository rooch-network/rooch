// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

// Copyright 2020 Parity Technologies
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

use fixed_hash::construct_fixed_hash;
use schemars::JsonSchema;
use serde::{Deserialize, Deserializer, Serialize, Serializer};

#[cfg(feature = "codec")]
use impl_codec::impl_fixed_hash_codec;
#[cfg(feature = "rlp")]
use impl_rlp::impl_fixed_hash_rlp;
#[cfg(feature = "serialize")]
use impl_serde::impl_fixed_hash_serde;

// 3 according to yellowpaper
// const BLOOM_BITS: u32 = 3;
const BLOOM_SIZE: usize = 256;

construct_fixed_hash! {
    /// Bloom hash type with 256 bytes (2048 bits) size.
    #[cfg_attr(feature = "codec", derive(scale_info::TypeInfo))]
    pub struct Bloom(BLOOM_SIZE);
}

impl Serialize for Bloom {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        // Serialize the byte array directly.
        serializer.serialize_bytes(&self.0)
    }
}

impl JsonSchema for Bloom {
    fn schema_name() -> String {
        // Provide a unique name for the schema (e.g., "Bloom").
        "Bloom".to_owned()
    }

    fn json_schema(_gen: &mut schemars::gen::SchemaGenerator) -> schemars::schema::Schema {
        // Define the JSON schema for the Bloom struct.
        // You need to specify the schema details here according to your requirements.
        // For example, if you want to represent it as a byte array:
        schemars::schema::SchemaObject {
            instance_type: Some(schemars::schema::InstanceType::String.into()),
            format: Some("byte".to_owned()),
            ..Default::default()
        }
        .into()
    }
}

impl From<&[u8]> for Bloom {
    fn from(bytes: &[u8]) -> Self {
        // Convert the byte slice to your custom `Bloom` struct.
        // You'll need to implement the logic to convert the bytes here.
        // For example:
        let mut data = [0u8; BLOOM_SIZE];
        data.copy_from_slice(bytes);
        Bloom(data)
    }
}

impl<'de> Deserialize<'de> for Bloom {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        // Deserialize as a byte slice.
        let bytes: &[u8] = Deserialize::deserialize(deserializer)?;

        // Convert the byte slice to your custom `Bloom` struct.
        let bloom: Bloom = bytes.try_into().map_err(|_| {
            serde::de::Error::custom("Failed to convert byte slice to Bloom struct")
        })?;

        Ok(bloom)
    }
}
