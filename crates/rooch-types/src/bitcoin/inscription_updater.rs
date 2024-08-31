// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use super::ord::{InscriptionID, SatPoint};
use framework_types::addresses::BITCOIN_MOVE_ADDRESS;
use move_core_types::{account_address::AccountAddress, ident_str, identifier::IdentStr};
use moveos_types::{
    move_std::option::MoveOption,
    state::{MoveState, MoveStructState, MoveStructType},
};
use serde::{Deserialize, Serialize};

pub const MODULE_NAME: &IdentStr = ident_str!("inscription_updater");

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum InscriptionUpdaterEvent {
    InscriptionCreated(InscriptionCreatedEvent),
    InscriptionTransferred(InscriptionTransferredEvent),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InscriptionCreatedEvent {
    pub block_height: u64,
    pub charms: u16,
    pub inscription_id: InscriptionID,
    pub location: MoveOption<SatPoint>,
    pub parent_inscription_ids: Vec<InscriptionID>,
    pub sequence_number: u32,
}

impl MoveStructType for InscriptionCreatedEvent {
    const ADDRESS: AccountAddress = BITCOIN_MOVE_ADDRESS;
    const MODULE_NAME: &'static IdentStr = MODULE_NAME;
    const STRUCT_NAME: &'static IdentStr = ident_str!("InscriptionCreatedEvent");
}

impl MoveStructState for InscriptionCreatedEvent {
    fn struct_layout() -> move_core_types::value::MoveStructLayout {
        move_core_types::value::MoveStructLayout::new(vec![
            move_core_types::value::MoveTypeLayout::U64,
            move_core_types::value::MoveTypeLayout::U16,
            InscriptionID::type_layout(),
            MoveOption::<SatPoint>::type_layout(),
            move_core_types::value::MoveTypeLayout::Vector(Box::new(InscriptionID::type_layout())),
            move_core_types::value::MoveTypeLayout::U32,
        ])
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InscriptionTransferredEvent {
    pub block_height: u64,
    pub inscription_id: InscriptionID,
    pub new_location: SatPoint,
    pub old_location: SatPoint,
    pub sequence_number: u32,
    pub is_burned: bool,
}

impl MoveStructType for InscriptionTransferredEvent {
    const ADDRESS: AccountAddress = BITCOIN_MOVE_ADDRESS;
    const MODULE_NAME: &'static IdentStr = MODULE_NAME;
    const STRUCT_NAME: &'static IdentStr = ident_str!("InscriptionTransferredEvent");
}

impl MoveStructState for InscriptionTransferredEvent {
    fn struct_layout() -> move_core_types::value::MoveStructLayout {
        move_core_types::value::MoveStructLayout::new(vec![
            move_core_types::value::MoveTypeLayout::U64,
            InscriptionID::type_layout(),
            SatPoint::type_layout(),
            SatPoint::type_layout(),
            move_core_types::value::MoveTypeLayout::U32,
            move_core_types::value::MoveTypeLayout::Bool,
        ])
    }
}
