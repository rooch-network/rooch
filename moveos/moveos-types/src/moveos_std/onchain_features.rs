// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use crate::addresses::MOVEOS_STD_ADDRESS;
use crate::moveos_std::object;
use crate::moveos_std::object::ObjectID;
use crate::state::{MoveStructState, MoveStructType};
use move_core_types::account_address::AccountAddress;
use move_core_types::ident_str;
use move_core_types::identifier::IdentStr;
use move_core_types::value::{MoveStructLayout, MoveTypeLayout};
use serde::{Deserialize, Serialize};

pub const MODULE_NAME: &IdentStr = ident_str!("features");
pub const VALUE_SIZE_GAS_FEATURE: u64 = 7;

#[derive(Clone, Debug, Deserialize, Serialize, Eq, PartialEq)]
pub struct FeatureStore {
    pub entries: Vec<u8>,
}

impl FeatureStore {
    pub fn feature_store_object_id() -> ObjectID {
        object::named_object_id(&Self::struct_tag())
    }

    pub fn contains_feature(&self, feature: u64) -> bool {
        let byte_index = feature / 8;
        let bit_mask = 1 << ((feature % 8) as u8);
        let value = self.entries[byte_index as usize];
        byte_index < self.entries.len() as u64 && (value & bit_mask) != 0
    }

    pub fn has_value_size_gas_feature(&self) -> bool {
        self.contains_feature(VALUE_SIZE_GAS_FEATURE)
    }
}

impl MoveStructType for FeatureStore {
    const ADDRESS: AccountAddress = MOVEOS_STD_ADDRESS;
    const MODULE_NAME: &'static IdentStr = MODULE_NAME;
    const STRUCT_NAME: &'static IdentStr = ident_str!("FeatureStore");
}

impl MoveStructState for FeatureStore {
    fn struct_layout() -> MoveStructLayout {
        MoveStructLayout::new(vec![MoveTypeLayout::Vector(Box::new(MoveTypeLayout::U8))])
    }
}
