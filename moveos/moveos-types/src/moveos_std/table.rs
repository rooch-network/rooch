// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use crate::{
    addresses::MOVEOS_STD_ADDRESS,
    state::{MoveStructState, MoveStructType},
};
use move_core_types::{
    account_address::AccountAddress,
    ident_str,
    identifier::IdentStr,
    language_storage::StructTag,
    value::{MoveStructLayout, MoveTypeLayout},
};
use serde::{Deserialize, Serialize};

pub const MODULE_NAME: &IdentStr = ident_str!("table");

#[derive(Eq, PartialEq, Debug, Clone, Deserialize, Serialize, Default)]
pub struct TablePlaceholder {
    // Move VM will auto add a bool field to the empty struct
    // So we manually add a bool field to the struct
    pub _placeholder: bool,
}

impl MoveStructType for TablePlaceholder {
    const ADDRESS: AccountAddress = MOVEOS_STD_ADDRESS;
    const MODULE_NAME: &'static IdentStr = MODULE_NAME;
    const STRUCT_NAME: &'static IdentStr = ident_str!("TablePlaceholder");

    fn struct_tag() -> StructTag {
        StructTag {
            address: Self::ADDRESS,
            module: Self::MODULE_NAME.to_owned(),
            name: Self::STRUCT_NAME.to_owned(),
            type_params: vec![],
        }
    }
}

impl MoveStructState for TablePlaceholder {
    fn struct_layout() -> MoveStructLayout {
        MoveStructLayout::new(vec![MoveTypeLayout::Bool])
    }
}

// /// Type of tables
// #[derive(Debug, Clone, Serialize, Deserialize)]
// pub struct Table<K, V> {
//     pub handle: Object<TablePlaceholder>,
//     pub k: std::marker::PhantomData<K>,
//     pub v: std::marker::PhantomData<V>,
// }
//
// impl<K, V> Table<K, V> {
//     pub fn new(handle: Object<TablePlaceholder>) -> Self {
//         Table {
//             handle,
//             k: std::marker::PhantomData,
//             v: std::marker::PhantomData,
//         }
//     }
// }
//
// impl<K, V> MoveStructType for Table<K, V>
// where
//     K: MoveStructType,
//     V: MoveStructType,
// {
//     const ADDRESS: AccountAddress = MOVEOS_STD_ADDRESS;
//     const MODULE_NAME: &'static IdentStr = MODULE_NAME;
//     const STRUCT_NAME: &'static IdentStr = ident_str!("Table");
//
//     fn struct_tag() -> StructTag {
//         StructTag {
//             address: Self::ADDRESS,
//             module: Self::MODULE_NAME.to_owned(),
//             name: Self::STRUCT_NAME.to_owned(),
//             type_params: vec![K::struct_tag().into(), V::struct_tag().into()],
//         }
//     }
// }
//
// impl<K, V> MoveStructState for Table<K, V>
// where
//     K: MoveStructType,
//     V: MoveStructType,
// {
//     fn struct_layout() -> MoveStructLayout {
//         MoveStructLayout::new(vec![MoveTypeLayout::Struct(
//             Object::<TablePlaceholder>::struct_layout(),
//         )])
//     }
// }
