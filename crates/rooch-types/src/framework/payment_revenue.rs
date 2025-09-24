// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use crate::addresses::ROOCH_FRAMEWORK_ADDRESS;
use move_core_types::language_storage::{StructTag, TypeTag};
use move_core_types::u256::U256;
use move_core_types::value::MoveValue;
use move_core_types::{account_address::AccountAddress, ident_str, identifier::IdentStr};
use moveos_types::module_binding::{ModuleBinding, MoveFunctionCaller};
use moveos_types::moveos_std::object::ObjectID;
use moveos_types::state::{MoveState, MoveStructState, MoveStructType};
use moveos_types::transaction::MoveAction;
use serde::{Deserialize, Serialize};

pub const MODULE_NAME: &IdentStr = ident_str!("payment_revenue");

/// PaymentRevenueHub structure for Rust binding
/// Matches the PaymentRevenueHub struct in payment_revenue.move
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PaymentRevenueHub {
    pub multi_coin_store: ObjectID, // Object<MultiCoinStore> stored as ObjectID
    pub revenue_by_source: ObjectID, // Table<String, Table<String, u256>> handle stored as ObjectID
}

impl MoveStructType for PaymentRevenueHub {
    const ADDRESS: AccountAddress = ROOCH_FRAMEWORK_ADDRESS;
    const MODULE_NAME: &'static IdentStr = MODULE_NAME;
    const STRUCT_NAME: &'static IdentStr = ident_str!("PaymentRevenueHub");

    fn struct_tag() -> StructTag {
        StructTag {
            address: Self::ADDRESS,
            module: Self::MODULE_NAME.to_owned(),
            name: Self::STRUCT_NAME.to_owned(),
            type_params: vec![],
        }
    }
}

impl MoveStructState for PaymentRevenueHub {
    fn struct_layout() -> move_core_types::value::MoveStructLayout {
        move_core_types::value::MoveStructLayout::new(vec![
            // multi_coin_store: Object<MultiCoinStore>
            ObjectID::type_layout(),
            // revenue_by_source: Table<String, Table<String, u256>>
            ObjectID::type_layout(),
        ])
    }
}

impl PaymentRevenueHub {
    pub fn multi_coin_store(&self) -> ObjectID {
        self.multi_coin_store.clone()
    }

    pub fn revenue_by_source(&self) -> ObjectID {
        self.revenue_by_source.clone()
    }
}

/// Rust bindings for rooch_framework::payment_revenue module
pub struct PaymentRevenueModule<'a> {
    _caller: &'a dyn MoveFunctionCaller,
}

impl<'a> PaymentRevenueModule<'a> {
    pub const WITHDRAW_REVENUE_ENTRY_FUNCTION_NAME: &'static IdentStr =
        ident_str!("withdraw_revenue_entry");

    /// Calculate the ObjectID for a payment revenue hub
    /// This replicates the logic from payment_revenue.move::get_payment_revenue_hub_id
    pub fn payment_revenue_hub_id(owner: AccountAddress) -> ObjectID {
        moveos_types::moveos_std::object::account_named_object_id(
            owner,
            &PaymentRevenueHub::struct_tag(),
        )
    }

    pub fn withdraw_revenue_entry_action(coin_type: StructTag, amount: U256) -> MoveAction {
        Self::create_move_action(
            Self::WITHDRAW_REVENUE_ENTRY_FUNCTION_NAME,
            vec![TypeTag::Struct(Box::new(coin_type))],
            vec![MoveValue::U256(amount)],
        )
    }
}

impl<'a> ModuleBinding<'a> for PaymentRevenueModule<'a> {
    const MODULE_NAME: &'static IdentStr = MODULE_NAME;
    const MODULE_ADDRESS: AccountAddress = ROOCH_FRAMEWORK_ADDRESS;

    fn new(caller: &'a impl MoveFunctionCaller) -> Self
    where
        Self: Sized,
    {
        Self { _caller: caller }
    }
}
