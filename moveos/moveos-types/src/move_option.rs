// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use crate::{
    addresses::MOVE_STD_ADDRESS,
    state::{MoveState, MoveStructState, MoveStructType, MoveType},
};
use move_core_types::{
    account_address::AccountAddress,
    ident_str,
    identifier::IdentStr,
    language_storage::TypeTag,
    value::{MoveStructLayout, MoveTypeLayout},
};
use serde::{Deserialize, Serialize};

/// `MoveOption` is represented `std::option::Option` in Move.
#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
pub struct MoveOption<E> {
    pub vec: Vec<E>,
}

impl<E> MoveOption<E> {
    pub fn some(e: E) -> Self {
        Self { vec: vec![e] }
    }

    pub fn none() -> Self {
        Self { vec: vec![] }
    }
}

impl<E> MoveStructType for MoveOption<E>
where
    E: MoveType,
{
    const ADDRESS: AccountAddress = MOVE_STD_ADDRESS;
    const MODULE_NAME: &'static IdentStr = ident_str!("option");
    const STRUCT_NAME: &'static IdentStr = ident_str!("Option");

    fn type_params() -> Vec<TypeTag> {
        vec![E::type_tag()]
    }
}

impl<E> MoveStructState for MoveOption<E>
where
    E: MoveState + Serialize,
{
    fn struct_layout() -> move_core_types::value::MoveStructLayout {
        MoveStructLayout::new(vec![MoveTypeLayout::Vector(Box::new(E::type_layout()))])
    }
}

impl<E> From<MoveOption<E>> for Option<E> {
    fn from(mut mv: MoveOption<E>) -> Self {
        if mv.vec.is_empty() {
            None
        } else {
            Some(mv.vec.pop().expect("MoveOption is not empty"))
        }
    }
}

impl<E> From<Option<E>> for MoveOption<E> {
    fn from(op: Option<E>) -> Self {
        match op {
            Some(e) => Self::some(e),
            None => Self::none(),
        }
    }
}
