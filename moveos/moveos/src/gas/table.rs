// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use crate::gas::gas_member::{FromOnChainGasSchedule, InitialGasSchedule, ToOnChainGasSchedule};
use crate::gas::r#abstract::{
    AbstractValueSize, AbstractValueSizePerArg, InternalGasPerAbstractValueUnit,
};
use move_binary_format::errors::{PartialVMError, PartialVMResult};
use move_binary_format::file_format::CodeOffset;
use move_core_types::account_address::AccountAddress;
use move_core_types::effects::Op;
use move_core_types::gas_algebra::{
    AbstractMemorySize, GasQuantity, InternalGas, InternalGasPerArg, InternalGasPerByte, NumArgs,
    NumBytes,
};
use move_core_types::language_storage::ModuleId;
use move_core_types::vm_status::StatusCode;
use move_vm_types::gas::{GasMeter, SimpleInstruction};
use move_vm_types::views::{TypeView, ValueView};
use moveos_types::moveos_std::event::TransactionEvent;
use moveos_types::state::StateChangeSet;
use moveos_types::transaction::GasStatement;
use once_cell::sync::Lazy;
use serde::{Deserialize, Serialize};
use std::cell::RefCell;
use std::collections::BTreeMap;
use std::ops::{Add, Bound};
use std::rc::Rc;

use super::SwitchableGasMeter;

/// The size in bytes for a reference on the stack
pub const REFERENCE_SIZE: AbstractMemorySize = AbstractMemorySize::new(8);

/// The size of a struct in bytes
pub const STRUCT_SIZE: AbstractMemorySize = AbstractMemorySize::new(2);

/// The size of a vector (without its containing data) in bytes
pub const VEC_SIZE: AbstractMemorySize = AbstractMemorySize::new(8);

pub const INSTRUCTION_TIER_DEFAULT: u64 = 1;
pub const STACK_HEIGHT_TIER_DEFAULT: u64 = 1;
pub const STACK_SIZE_TIER_DEFAULT: u64 = 1;

pub static ZERO_COST_SCHEDULE: Lazy<CostTable> = Lazy::new(zero_cost_schedule);

#[derive(Clone, Debug, Default, Serialize, PartialEq, Eq, Deserialize)]
pub struct StorageGasParameter {
    pub io_read_price: u64,
    pub storage_fee_per_transaction_byte: u64,
    pub storage_fee_per_event_byte: u64,
    pub storage_fee_per_op_new_byte: u64,
    pub storage_fee_per_op_modify_byte: u64,
    pub storage_fee_per_op_delete: u64,
}

#[derive(Clone, Debug, Serialize, PartialEq, Eq, Deserialize)]
pub struct AbstractValueSizeGasParameter {
    pub u8: AbstractValueSize,
    pub u16: AbstractValueSize,
    pub u32: AbstractValueSize,
    pub u64: AbstractValueSize,
    pub u128: AbstractValueSize,
    pub u256: AbstractValueSize,
    pub bool: AbstractValueSize,
    pub address: AbstractValueSize,
    pub struct_: AbstractValueSize,
    pub vector: AbstractValueSize,
    pub reference: AbstractValueSize,
    pub per_u8_packed: AbstractValueSizePerArg,
    pub per_u16_packed: AbstractValueSizePerArg,
    pub per_u32_packed: AbstractValueSizePerArg,
    pub per_u64_packed: AbstractValueSizePerArg,
    pub per_u128_packed: AbstractValueSizePerArg,
    pub per_u256_packed: AbstractValueSizePerArg,
    pub per_bool_packed: AbstractValueSizePerArg,
    pub per_address_packed: AbstractValueSizePerArg,
}

impl AbstractValueSizeGasParameter {
    pub fn zeros() -> Self {
        Self {
            u8: 0.into(),
            u16: 0.into(),
            u32: 0.into(),
            u64: 0.into(),
            u128: 0.into(),
            u256: 0.into(),
            bool: 0.into(),
            address: 0.into(),
            struct_: 0.into(),
            vector: 0.into(),
            reference: 0.into(),
            per_u8_packed: 0.into(),
            per_u16_packed: 0.into(),
            per_u32_packed: 0.into(),
            per_u64_packed: 0.into(),
            per_u128_packed: 0.into(),
            per_u256_packed: 0.into(),
            per_bool_packed: 0.into(),
            per_address_packed: 0.into(),
        }
    }
}

impl StorageGasParameter {
    pub fn zeros() -> Self {
        Self {
            io_read_price: 0,
            storage_fee_per_transaction_byte: 0,
            storage_fee_per_event_byte: 0,
            storage_fee_per_op_new_byte: 0,
            storage_fee_per_op_modify_byte: 0,
            storage_fee_per_op_delete: 0,
        }
    }
}

#[derive(Clone, Debug, Serialize, PartialEq, Eq, Deserialize)]
pub struct InstructionParameter {
    pub nop: InternalGas,
    pub ret: InternalGas,
    pub abort: InternalGas,
    pub br_true: InternalGas,
    pub br_false: InternalGas,
    pub branch: InternalGas,
    pub pop: InternalGas,
    pub ld_u8: InternalGas,
    pub ld_u16: InternalGas,
    pub ld_u32: InternalGas,
    pub ld_u64: InternalGas,
    pub ld_u128: InternalGas,
    pub ld_u256: InternalGas,
    pub ld_true: InternalGas,
    pub ld_false: InternalGas,
    pub ld_const_base: InternalGas,
    pub ld_const_per_byte: InternalGasPerByte,
    pub imm_borrow_loc: InternalGas,
    pub mut_borrow_loc: InternalGas,
    pub imm_borrow_field: InternalGas,
    pub mut_borrow_field: InternalGas,
    pub imm_borrow_field_generic: InternalGas,
    pub mut_borrow_field_generic: InternalGas,
    pub copy_loc_base: InternalGas,
    pub copy_loc_per_abs_val_unit: InternalGasPerAbstractValueUnit,
    pub move_loc_base: InternalGas,
    pub st_loc_base: InternalGas,
    pub call_base: InternalGas,
    pub call_per_arg: InternalGasPerArg,
    pub call_per_local: InternalGasPerArg,
    pub call_generic_base: InternalGas,
    pub call_generic_per_ty_arg: InternalGasPerArg,
    pub call_generic_per_arg: InternalGasPerArg,
    pub call_generic_per_local: InternalGasPerArg,
    pub pack_base: InternalGas,
    pub pack_per_field: InternalGasPerArg,
    pub pack_generic_base: InternalGas,
    pub pack_generic_per_field: InternalGasPerArg,
    pub unpack_base: InternalGas,
    pub unpack_per_field: InternalGasPerArg,
    pub unpack_generic_base: InternalGas,
    pub unpack_generic_per_field: InternalGasPerArg,
    pub read_ref_base: InternalGas,
    pub read_ref_per_abs_val_unit: InternalGasPerAbstractValueUnit,
    pub write_ref_base: InternalGas,
    pub freeze_ref: InternalGas,
    pub cast_u8: InternalGas,
    pub cast_u16: InternalGas,
    pub cast_u32: InternalGas,
    pub cast_u64: InternalGas,
    pub cast_u128: InternalGas,
    pub cast_u256: InternalGas,
    pub add: InternalGas,
    pub sub: InternalGas,
    pub mul: InternalGas,
    pub mod_: InternalGas,
    pub div: InternalGas,
    pub bit_or: InternalGas,
    pub bit_and: InternalGas,
    pub xor: InternalGas,
    pub shl: InternalGas,
    pub shr: InternalGas,
    pub or: InternalGas,
    pub and: InternalGas,
    pub not: InternalGas,
    pub lt: InternalGas,
    pub gt: InternalGas,
    pub le: InternalGas,
    pub ge: InternalGas,
    pub eq_base: InternalGas,
    pub eq_per_abs_val_unit: InternalGasPerAbstractValueUnit,
    pub neq_base: InternalGas,
    pub neq_per_abs_val_unit: InternalGasPerAbstractValueUnit,
    pub imm_borrow_global_base: InternalGas,
    pub imm_borrow_global_generic_base: InternalGas,
    pub mut_borrow_global_base: InternalGas,
    pub mut_borrow_global_generic_base: InternalGas,
    pub exists_base: InternalGas,
    pub exists_generic_base: InternalGas,
    pub move_from_base: InternalGas,
    pub move_from_generic_base: InternalGas,
    pub move_to_base: InternalGas,
    pub move_to_generic_base: InternalGas,
    pub vec_len_base: InternalGas,
    pub vec_imm_borrow_base: InternalGas,
    pub vec_mut_borrow_base: InternalGas,
    pub vec_push_back_base: InternalGas,
    pub vec_pop_back_base: InternalGas,
    pub vec_swap_base: InternalGas,
    pub vec_pack_base: InternalGas,
    pub vec_pack_per_elem: InternalGasPerArg,
    pub vec_unpack_base: InternalGas,
    pub vec_unpack_per_expected_elem: InternalGasPerArg,
}

impl InstructionParameter {
    pub fn zeros() -> Self {
        Self {
            nop: 0.into(),
            ret: 0.into(),
            abort: 0.into(),
            br_true: 0.into(),
            br_false: 0.into(),
            branch: 0.into(),
            pop: 0.into(),
            ld_u8: 0.into(),
            ld_u16: 0.into(),
            ld_u32: 0.into(),
            ld_u64: 0.into(),
            ld_u128: 0.into(),
            ld_u256: 0.into(),
            ld_true: 0.into(),
            ld_false: 0.into(),
            ld_const_base: 0.into(),
            ld_const_per_byte: 0.into(),
            imm_borrow_loc: 0.into(),
            mut_borrow_loc: 0.into(),
            imm_borrow_field: 0.into(),
            mut_borrow_field: 0.into(),
            imm_borrow_field_generic: 0.into(),
            mut_borrow_field_generic: 0.into(),
            copy_loc_base: 0.into(),
            copy_loc_per_abs_val_unit: 0.into(),
            move_loc_base: 0.into(),
            st_loc_base: 0.into(),
            call_base: 0.into(),
            call_per_arg: 0.into(),
            call_per_local: 0.into(),
            call_generic_base: 0.into(),
            call_generic_per_ty_arg: 0.into(),
            call_generic_per_arg: 0.into(),
            call_generic_per_local: 0.into(),
            pack_base: 0.into(),
            pack_per_field: 0.into(),
            pack_generic_base: 0.into(),
            pack_generic_per_field: 0.into(),
            unpack_base: 0.into(),
            unpack_per_field: 0.into(),
            unpack_generic_base: 0.into(),
            unpack_generic_per_field: 0.into(),
            read_ref_base: 0.into(),
            read_ref_per_abs_val_unit: 0.into(),
            write_ref_base: 0.into(),
            freeze_ref: 0.into(),
            cast_u8: 0.into(),
            cast_u16: 0.into(),
            cast_u32: 0.into(),
            cast_u64: 0.into(),
            cast_u128: 0.into(),
            cast_u256: 0.into(),
            add: 0.into(),
            sub: 0.into(),
            mul: 0.into(),
            mod_: 0.into(),
            div: 0.into(),
            bit_or: 0.into(),
            bit_and: 0.into(),
            xor: 0.into(),
            shl: 0.into(),
            shr: 0.into(),
            or: 0.into(),
            and: 0.into(),
            not: 0.into(),
            lt: 0.into(),
            gt: 0.into(),
            le: 0.into(),
            ge: 0.into(),
            eq_base: 0.into(),
            eq_per_abs_val_unit: 0.into(),
            neq_base: 0.into(),
            neq_per_abs_val_unit: 0.into(),
            imm_borrow_global_base: 0.into(),
            imm_borrow_global_generic_base: 0.into(),
            mut_borrow_global_base: 0.into(),
            mut_borrow_global_generic_base: 0.into(),
            exists_base: 0.into(),
            exists_generic_base: 0.into(),
            move_from_base: 0.into(),
            move_from_generic_base: 0.into(),
            move_to_base: 0.into(),
            move_to_generic_base: 0.into(),
            vec_len_base: 0.into(),
            vec_imm_borrow_base: 0.into(),
            vec_mut_borrow_base: 0.into(),
            vec_push_back_base: 0.into(),
            vec_pop_back_base: 0.into(),
            vec_swap_base: 0.into(),
            vec_pack_base: 0.into(),
            vec_pack_per_elem: 0.into(),
            vec_unpack_base: 0.into(),
            vec_unpack_per_expected_elem: 0.into(),
        }
    }
}

#[derive(Clone, Debug, Serialize, PartialEq, Eq, Deserialize)]
pub struct CostTable {
    pub instruction_tiers: BTreeMap<u64, u64>,
    pub stack_height_tiers: BTreeMap<u64, u64>,
    pub stack_size_tiers: BTreeMap<u64, u64>,
    pub storage_gas_parameter: StorageGasParameter,
    pub instruction_gas_parameter: InstructionParameter,
    pub abstract_value_parameter: AbstractValueSizeGasParameter,
}

impl CostTable {
    fn get_current_and_future_tier(
        tiers: &BTreeMap<u64, u64>,
        current: u64,
        default: u64,
    ) -> (u64, Option<u64>) {
        let current_cost = tiers
            .get(&current)
            .or_else(|| tiers.range(..current).next_back().map(|(_, v)| v))
            .unwrap_or(&default);
        let next_tier_start = tiers
            .range::<u64, _>((Bound::Excluded(current), Bound::Unbounded))
            .next()
            .map(|(next_tier_start, _)| *next_tier_start);
        (*current_cost, next_tier_start)
    }

    pub fn instruction_tier(&self, instr_count: u64) -> (u64, Option<u64>) {
        Self::get_current_and_future_tier(
            &self.instruction_tiers,
            instr_count,
            INSTRUCTION_TIER_DEFAULT,
        )
    }

    pub fn stack_height_tier(&self, stack_height: u64) -> (u64, Option<u64>) {
        Self::get_current_and_future_tier(
            &self.stack_height_tiers,
            stack_height,
            STACK_HEIGHT_TIER_DEFAULT,
        )
    }

    pub fn stack_size_tier(&self, stack_size: u64) -> (u64, Option<u64>) {
        Self::get_current_and_future_tier(
            &self.stack_size_tiers,
            stack_size,
            STACK_SIZE_TIER_DEFAULT,
        )
    }
}

/// The  `GasCost` tracks:
/// - instruction cost: how much time/computational power is needed to perform the instruction
/// - memory cost: how much memory is required for the instruction, and storage overhead
/// - stack height: how high is the stack growing (regardless of size in bytes)
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct GasCost {
    pub instruction_gas: u64,
    pub memory_gas: u64,
    pub stack_height_gas: u64,
}

pub fn initial_cost_schedule() -> CostTable {
    let instruction_tiers: BTreeMap<u64, u64> = vec![
        (0, 1),
        (3000, 2),
        (6000, 3),
        (8000, 5),
        (9000, 9),
        (9500, 16),
        (10000, 29),
        (10500, 50),
        (15000, 100),
    ]
    .into_iter()
    .collect();

    let stack_height_tiers: BTreeMap<u64, u64> = vec![
        (0, 1),
        (400, 2),
        (800, 3),
        (1200, 5),
        (1500, 9),
        (1800, 16),
        (2000, 29),
        (2200, 50),
        (5000, 100),
    ]
    .into_iter()
    .collect();

    let stack_size_tiers: BTreeMap<u64, u64> = vec![
        (0, 1),
        (2000, 2),
        (5000, 3),
        (8000, 5),
        (10000, 9),
        (11000, 16),
        (11500, 29),
        (11500, 50),
        (20000, 100),
    ]
    .into_iter()
    .collect();

    let storage_gas_parameter = StorageGasParameter::initial();
    let instruction_gas_parameter = InstructionParameter::initial();
    let abstract_value_gas_parameter = AbstractValueSizeGasParameter::initial();

    CostTable {
        instruction_tiers,
        stack_size_tiers,
        stack_height_tiers,
        storage_gas_parameter,
        instruction_gas_parameter,
        abstract_value_parameter: abstract_value_gas_parameter,
    }
}

pub fn zero_cost_schedule() -> CostTable {
    let mut zero_tier = BTreeMap::new();
    zero_tier.insert(0, 0);
    CostTable {
        instruction_tiers: zero_tier.clone(),
        stack_size_tiers: zero_tier.clone(),
        stack_height_tiers: zero_tier,
        storage_gas_parameter: StorageGasParameter::default(),
        instruction_gas_parameter: InstructionParameter::zeros(),
        abstract_value_parameter: AbstractValueSizeGasParameter::zeros(),
    }
}

impl GasCost {
    pub fn new(instruction_gas: u64, memory_gas: u64, stack_height_gas: u64) -> Self {
        Self {
            instruction_gas,
            memory_gas,
            stack_height_gas,
        }
    }

    /// Convert a GasCost to a total gas charge in `InternalGas`.
    #[inline]
    pub fn total(&self) -> u64 {
        self.instruction_gas
            .add(self.memory_gas)
            .add(self.stack_height_gas)
    }

    #[inline]
    pub fn total_internal(&self) -> InternalGas {
        GasQuantity::new(
            self.instruction_gas
                .add(self.memory_gas)
                .add(self.stack_height_gas),
        )
    }
}

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct MoveOSGasMeter {
    cost_table: CostTable,
    gas_left: InternalGas,
    //TODO we do not need to use gas_price in gas meter.
    charge: bool,

    execution_gas_used: Rc<RefCell<InternalGas>>,
    storage_gas_used: Rc<RefCell<InternalGas>>,

    // The current height of the operand stack, and the maximal height that it has reached.
    stack_height_high_water_mark: u64,
    stack_height_current: u64,
    stack_height_next_tier_start: Option<u64>,
    stack_height_current_tier_mult: u64,

    // The current (abstract) size  of the operand stack and the maximal size that it has reached.
    stack_size_high_water_mark: u64,
    stack_size_current: u64,
    stack_size_next_tier_start: Option<u64>,
    stack_size_current_tier_mult: u64,

    // The total number of bytecode instructions that have been executed in the transaction.
    instructions_executed: u64,
    instructions_next_tier_start: Option<u64>,
    instructions_current_tier_mult: u64,
}

impl MoveOSGasMeter {
    /// Initialize the gas state with metering enabled.
    ///
    /// Charge for every operation and fail when there is no more gas to pay for operations.
    /// This is the instantiation that must be used when executing a user function.
    pub fn new(cost_table: CostTable, budget: u64) -> Self {
        //assert!(gas_price > 0, "gas price cannot be 0");
        //let budget_in_unit = budget / gas_price;
        // let gas_left = Self::to_internal_units(budget_in_unit);
        let (stack_height_current_tier_mult, stack_height_next_tier_start) =
            cost_table.stack_height_tier(0);
        let (stack_size_current_tier_mult, stack_size_next_tier_start) =
            cost_table.stack_size_tier(0);
        let (instructions_current_tier_mult, instructions_next_tier_start) =
            cost_table.instruction_tier(0);
        Self {
            gas_left: InternalGas::from(budget),
            cost_table,
            charge: true,
            execution_gas_used: Rc::new(RefCell::new(InternalGas::from(0))),
            storage_gas_used: Rc::new(RefCell::new(InternalGas::from(0))),
            stack_height_high_water_mark: 0,
            stack_height_current: 0,
            stack_size_high_water_mark: 0,
            stack_size_current: 0,
            instructions_executed: 0,
            stack_height_current_tier_mult,
            stack_size_current_tier_mult,
            instructions_current_tier_mult,
            stack_height_next_tier_start,
            stack_size_next_tier_start,
            instructions_next_tier_start,
        }
    }

    /// Initialize the gas state with metering disabled.
    ///
    /// It should be used by clients in very specific cases and when executing system
    /// code that does not have to charge the user.
    pub fn new_unmetered() -> Self {
        Self {
            cost_table: ZERO_COST_SCHEDULE.clone(),
            gas_left: InternalGas::from(0),
            charge: false,
            execution_gas_used: Rc::new(RefCell::new(InternalGas::from(0))),
            storage_gas_used: Rc::new(RefCell::new(InternalGas::from(0))),
            stack_height_high_water_mark: 0,
            stack_height_current: 0,
            stack_height_next_tier_start: None,
            stack_height_current_tier_mult: 0,
            stack_size_high_water_mark: 0,
            stack_size_current: 0,
            stack_size_next_tier_start: None,
            stack_size_current_tier_mult: 0,
            instructions_executed: 0,
            instructions_next_tier_start: None,
            instructions_current_tier_mult: 0,
        }
    }

    pub fn push_stack(&mut self, pushes: u64) -> PartialVMResult<()> {
        match self.stack_height_current.checked_add(pushes) {
            // We should never hit this.
            None => return Err(PartialVMError::new(StatusCode::ARITHMETIC_ERROR)),
            Some(new_height) => {
                if new_height > self.stack_height_high_water_mark {
                    self.stack_height_high_water_mark = new_height;
                }
                self.stack_height_current = new_height;
            }
        }

        if let Some(stack_height_tier_next) = self.stack_height_next_tier_start {
            if self.stack_height_current > stack_height_tier_next {
                let (next_mul, next_tier) =
                    self.cost_table.stack_height_tier(self.stack_height_current);
                self.stack_height_current_tier_mult = next_mul;
                self.stack_height_next_tier_start = next_tier;
            }
        }

        Ok(())
    }

    pub fn increase_instruction_count(&mut self, amount: u64) -> PartialVMResult<()> {
        match self.instructions_executed.checked_add(amount) {
            None => return Err(PartialVMError::new(StatusCode::ARITHMETIC_ERROR)),
            Some(new_pc) => {
                self.instructions_executed = new_pc;
            }
        }

        if let Some(instr_tier_next) = self.instructions_next_tier_start {
            if self.instructions_executed > instr_tier_next {
                let (instr_cost, next_tier) =
                    self.cost_table.instruction_tier(self.instructions_executed);
                self.instructions_current_tier_mult = instr_cost;
                self.instructions_next_tier_start = next_tier;
            }
        }

        Ok(())
    }

    pub fn increase_stack_size(&mut self, size_amount: u64) -> PartialVMResult<()> {
        match self.stack_size_current.checked_add(size_amount) {
            None => return Err(PartialVMError::new(StatusCode::ARITHMETIC_ERROR)),
            Some(new_size) => {
                if new_size > self.stack_size_high_water_mark {
                    self.stack_size_high_water_mark = new_size;
                }
                self.stack_size_current = new_size;
            }
        }

        if let Some(stack_size_tier_next) = self.stack_size_next_tier_start {
            if self.stack_size_current > stack_size_tier_next {
                let (next_mul, next_tier) =
                    self.cost_table.stack_size_tier(self.stack_size_current);
                self.stack_size_current_tier_mult = next_mul;
                self.stack_size_next_tier_start = next_tier;
            }
        }

        Ok(())
    }

    pub fn pop_stack(&mut self, pops: u64) {
        self.stack_height_current = self.stack_height_current.saturating_sub(pops);
    }

    pub fn charge_v1(&mut self, cost: InternalGas) -> PartialVMResult<()> {
        self.deduct_gas(cost)
    }

    pub fn deduct_gas(&mut self, cost: InternalGas) -> PartialVMResult<()> {
        if !self.charge {
            return Ok(());
        }

        match self.gas_left.checked_sub(cost) {
            None => {
                self.gas_left = InternalGas::from(0);
                Err(PartialVMError::new(StatusCode::OUT_OF_GAS))
            }
            Some(gas_left) => {
                self.gas_left = gas_left;
                Ok(())
            }
        }
    }

    pub fn set_metering(&mut self, enabled: bool) {
        self.charge = enabled;
    }
}

pub trait ClassifiedGasMeter {
    fn charge_execution(&mut self, gas_cost: u64) -> PartialVMResult<()>;
    // fn charge_io_read(&mut self);
    fn charge_io_write(&mut self, data_size: u64) -> PartialVMResult<()>;
    fn charge_event(&mut self, events: &[TransactionEvent]) -> PartialVMResult<()>;
    fn charge_change_set(&mut self, change_set: &StateChangeSet) -> PartialVMResult<()>;
    fn check_constrains(&self, max_gas_amount: u64) -> PartialVMResult<()>;
    fn gas_statement(&self) -> GasStatement;
}

impl ClassifiedGasMeter for MoveOSGasMeter {
    fn charge_execution(&mut self, gas_cost: u64) -> PartialVMResult<()> {
        if !self.charge {
            return Ok(());
        }

        let new_value = self
            .execution_gas_used
            .borrow()
            .add(InternalGas::from(gas_cost));
        *self.execution_gas_used.borrow_mut() = new_value;
        Ok(())
    }

    // fn charge_io_read(&mut self) {}

    fn charge_io_write(&mut self, data_size: u64) -> PartialVMResult<()> {
        if !self.charge {
            return Ok(());
        }

        let fee = self
            .cost_table
            .storage_gas_parameter
            .storage_fee_per_transaction_byte
            * data_size;
        let new_value = self.storage_gas_used.borrow().add(InternalGas::from(fee));
        *self.storage_gas_used.borrow_mut() = new_value;
        self.deduct_gas(InternalGas::from(fee))
    }

    fn charge_event(&mut self, events: &[TransactionEvent]) -> PartialVMResult<()> {
        if !self.charge {
            return Ok(());
        }

        let mut total_event_fee = 0;
        for event in events {
            let fee = event.event_data.len() as u64
                * self
                    .cost_table
                    .storage_gas_parameter
                    .storage_fee_per_event_byte;
            let new_value = self.storage_gas_used.borrow().add(InternalGas::from(fee));
            *self.storage_gas_used.borrow_mut() = new_value;
            total_event_fee += fee;
        }
        self.deduct_gas(InternalGas::from(total_event_fee))
    }

    fn charge_change_set(&mut self, change_set: &StateChangeSet) -> PartialVMResult<()> {
        if !self.charge {
            return Ok(());
        }

        let mut total_change_set_fee = 0;
        for (_, table_change) in change_set.changes.iter() {
            for (key, op) in table_change.entries.iter() {
                let fee = {
                    match op {
                        Op::Modify(value) => {
                            (key.len() + value.value.len()) as u64
                                * self
                                    .cost_table
                                    .storage_gas_parameter
                                    .storage_fee_per_op_modify_byte
                        }
                        Op::Delete => {
                            self.cost_table
                                .storage_gas_parameter
                                .storage_fee_per_op_delete
                        }
                        Op::New(value) => {
                            (key.len() + value.value.len()) as u64
                                * self
                                    .cost_table
                                    .storage_gas_parameter
                                    .storage_fee_per_op_new_byte
                        }
                    }
                };
                let new_value = self.storage_gas_used.borrow().add(InternalGas::from(fee));
                *self.storage_gas_used.borrow_mut() = new_value;
                total_change_set_fee += fee;
            }
        }
        self.deduct_gas(InternalGas::from(total_change_set_fee))
    }

    fn check_constrains(&self, max_gas_amount: u64) -> PartialVMResult<()> {
        let gas_left: u64 = self.balance_internal().into();
        let gas_used = max_gas_amount.checked_sub(gas_left).unwrap_or_else(
            || panic!("gas_left({gas_left}) should always be less than or equal to max gas amount({max_gas_amount})")
        );

        if gas_used == 0 {
            return Ok(());
        }

        let execution_gas_used = *self.execution_gas_used.borrow();
        let storage_gas_used = *self.storage_gas_used.borrow();
        if InternalGas::from(gas_used) != execution_gas_used + storage_gas_used {
            return Err(PartialVMError::new(StatusCode::ABORTED)
                .with_message("Failed to check the constraints of the gas_used.".to_owned()));
        }
        Ok(())
    }

    fn gas_statement(&self) -> GasStatement {
        GasStatement {
            execution_gas_used: *self.execution_gas_used.borrow(),
            storage_gas_used: *self.storage_gas_used.borrow(),
        }
    }
}

impl GasMeter for MoveOSGasMeter {
    fn balance_internal(&self) -> InternalGas {
        self.gas_left
    }

    fn charge_simple_instr(&mut self, instr: SimpleInstruction) -> PartialVMResult<()> {
        macro_rules! dispatch {
            ($($name: ident => $cost: expr),* $(,)?) => {
                match instr {
                    $(SimpleInstruction::$name => self.deduct_gas($cost)),*
                }
            };
        }

        let instruction_gas_parameter = self.cost_table.instruction_gas_parameter.clone();

        dispatch! {
            Nop => instruction_gas_parameter.nop,

            Abort => instruction_gas_parameter.abort,
            Ret => instruction_gas_parameter.abort,

            LdU8 => instruction_gas_parameter.ld_u8,
            LdU16 => instruction_gas_parameter.ld_u16,
            LdU32 => instruction_gas_parameter.ld_u32,
            LdU64 => instruction_gas_parameter.ld_u64,
            LdU128 => instruction_gas_parameter.ld_u128,
            LdU256 => instruction_gas_parameter.ld_u256,
            LdTrue => instruction_gas_parameter.ld_true,
            LdFalse => instruction_gas_parameter.ld_false,

            ImmBorrowLoc => instruction_gas_parameter.imm_borrow_loc,
            MutBorrowLoc => instruction_gas_parameter.mut_borrow_loc,
            ImmBorrowField => instruction_gas_parameter.imm_borrow_field,
            MutBorrowField => instruction_gas_parameter.mut_borrow_field,
            ImmBorrowFieldGeneric => instruction_gas_parameter.imm_borrow_field_generic,
            MutBorrowFieldGeneric => instruction_gas_parameter.mut_borrow_field_generic,
            FreezeRef => instruction_gas_parameter.freeze_ref,

            CastU8 => instruction_gas_parameter.cast_u8,
            CastU16 => instruction_gas_parameter.cast_u16,
            CastU32 => instruction_gas_parameter.cast_u32,
            CastU64 => instruction_gas_parameter.cast_u64,
            CastU128 => instruction_gas_parameter.cast_u128,
            CastU256 => instruction_gas_parameter.cast_u256,

            Add => instruction_gas_parameter.add,
            Sub => instruction_gas_parameter.sub,
            Mul => instruction_gas_parameter.mul,
            Mod => instruction_gas_parameter.mod_,
            Div => instruction_gas_parameter.div,

            BitOr => instruction_gas_parameter.bit_or,
            BitAnd => instruction_gas_parameter.bit_and,
            Xor => instruction_gas_parameter.xor,
            Shl => instruction_gas_parameter.shl,
            Shr => instruction_gas_parameter.shr,

            Or => instruction_gas_parameter.or,
            And => instruction_gas_parameter.and,
            Not => instruction_gas_parameter.not,

            Lt => instruction_gas_parameter.lt,
            Gt => instruction_gas_parameter.gt,
            Le => instruction_gas_parameter.le,
            Ge => instruction_gas_parameter.ge,
        }
    }

    fn charge_br_true(&mut self, _target_offset: Option<CodeOffset>) -> PartialVMResult<()> {
        self.charge_v1(self.cost_table.instruction_gas_parameter.br_true)
    }

    fn charge_br_false(&mut self, _target_offset: Option<CodeOffset>) -> PartialVMResult<()> {
        self.charge_v1(self.cost_table.instruction_gas_parameter.br_false)
    }

    fn charge_branch(&mut self, _target_offset: CodeOffset) -> PartialVMResult<()> {
        self.charge_v1(self.cost_table.instruction_gas_parameter.branch)
    }

    fn charge_pop(&mut self, _popped_val: impl ValueView) -> PartialVMResult<()> {
        self.charge_v1(self.cost_table.instruction_gas_parameter.pop)
    }

    fn charge_call(
        &mut self,
        _module_id: &ModuleId,
        _func_name: &str,
        args: impl ExactSizeIterator<Item = impl ValueView>,
        num_locals: NumArgs,
    ) -> PartialVMResult<()> {
        let call_base = self.cost_table.instruction_gas_parameter.call_base;
        let call_per_arg = self.cost_table.instruction_gas_parameter.call_per_arg;
        let cost = call_base + call_per_arg * NumArgs::new(args.len() as u64);
        let call_per_local = self.cost_table.instruction_gas_parameter.call_per_local;
        self.charge_v1(cost + call_per_local * num_locals)
    }

    fn charge_call_generic(
        &mut self,
        _module_id: &ModuleId,
        _func_name: &str,
        ty_args: impl ExactSizeIterator<Item = impl TypeView>,
        args: impl ExactSizeIterator<Item = impl ValueView>,
        num_locals: NumArgs,
    ) -> PartialVMResult<()> {
        let call_generic_base = self.cost_table.instruction_gas_parameter.call_generic_base;
        let call_generic_per_type_arg = self
            .cost_table
            .instruction_gas_parameter
            .call_generic_per_ty_arg;
        let call_generic_per_arg = self
            .cost_table
            .instruction_gas_parameter
            .call_generic_per_arg;

        let cost = call_generic_base
            + call_generic_per_type_arg * NumArgs::new(ty_args.len() as u64)
            + call_generic_per_arg * NumArgs::new(args.len() as u64);

        let call_generic_per_local = self
            .cost_table
            .instruction_gas_parameter
            .call_generic_per_local;

        self.charge_v1(cost + call_generic_per_local * num_locals)
    }

    fn charge_ld_const(&mut self, size: NumBytes) -> PartialVMResult<()> {
        let ld_const_base = self.cost_table.instruction_gas_parameter.ld_const_base;
        let ld_const_per_byte = self.cost_table.instruction_gas_parameter.ld_const_per_byte;
        self.charge_v1(ld_const_base + ld_const_per_byte * size)
    }

    fn charge_ld_const_after_deserialization(
        &mut self,
        _val: impl ValueView,
    ) -> PartialVMResult<()> {
        // We already charged for this based on the bytes that we're loading so don't charge again.
        Ok(())
    }

    fn charge_copy_loc(&mut self, val: impl ValueView) -> PartialVMResult<()> {
        let (stack_size, heap_size) = self
            .cost_table
            .abstract_value_parameter
            .abstract_value_size_stack_and_heap(val);

        let copy_loc_base = self.cost_table.instruction_gas_parameter.copy_loc_base;
        let copy_loc_per_abs_val_unit = self
            .cost_table
            .instruction_gas_parameter
            .copy_loc_per_abs_val_unit;

        self.charge_v1(copy_loc_base + copy_loc_per_abs_val_unit * (stack_size + heap_size))
    }

    fn charge_move_loc(&mut self, _val: impl ValueView) -> PartialVMResult<()> {
        let move_local_base = self.cost_table.instruction_gas_parameter.move_loc_base;
        self.charge_v1(move_local_base)
    }

    fn charge_store_loc(&mut self, _val: impl ValueView) -> PartialVMResult<()> {
        let store_local_base = self.cost_table.instruction_gas_parameter.st_loc_base;
        self.charge_v1(store_local_base)
    }

    fn charge_pack(
        &mut self,
        is_generic: bool,
        args: impl ExactSizeIterator<Item = impl ValueView>,
    ) -> PartialVMResult<()> {
        let num_args = NumArgs::new(args.len() as u64);

        match is_generic {
            false => {
                let pack_base = self.cost_table.instruction_gas_parameter.pack_base;
                let pack_per_field = self.cost_table.instruction_gas_parameter.pack_per_field;
                self.charge_v1(pack_base + pack_per_field * num_args)
            }
            true => {
                let pack_generic_base = self.cost_table.instruction_gas_parameter.pack_generic_base;
                let pack_generic_per_field = self
                    .cost_table
                    .instruction_gas_parameter
                    .pack_generic_per_field;
                self.charge_v1(pack_generic_base + pack_generic_per_field * num_args)
            }
        }
    }

    fn charge_unpack(
        &mut self,
        is_generic: bool,
        args: impl ExactSizeIterator<Item = impl ValueView>,
    ) -> PartialVMResult<()> {
        let num_args = NumArgs::new(args.len() as u64);

        match is_generic {
            false => {
                let unpack_base = self.cost_table.instruction_gas_parameter.unpack_base;
                let unpack_per_field = self.cost_table.instruction_gas_parameter.unpack_per_field;
                self.charge_v1(unpack_base + unpack_per_field * num_args)
            }
            true => {
                let unpack_generic_base = self
                    .cost_table
                    .instruction_gas_parameter
                    .unpack_generic_base;
                let unpack_generic_per_field = self
                    .cost_table
                    .instruction_gas_parameter
                    .unpack_generic_per_field;
                self.charge_v1(unpack_generic_base + unpack_generic_per_field * num_args)
            }
        }
    }

    fn charge_read_ref(&mut self, ref_val: impl ValueView) -> PartialVMResult<()> {
        let (stack_size, heap_size) = self
            .cost_table
            .abstract_value_parameter
            .abstract_value_size_stack_and_heap(ref_val);

        let read_ref_base = self.cost_table.instruction_gas_parameter.read_ref_base;
        let read_ref_per_abs_val_unit = self
            .cost_table
            .instruction_gas_parameter
            .read_ref_per_abs_val_unit;

        self.charge_v1(read_ref_base + read_ref_per_abs_val_unit * (stack_size + heap_size))
    }

    fn charge_write_ref(
        &mut self,
        _new_val: impl ValueView,
        _old_val: impl ValueView,
    ) -> PartialVMResult<()> {
        let write_ref_base = self.cost_table.instruction_gas_parameter.write_ref_base;

        self.charge_v1(write_ref_base)
    }

    fn charge_eq(&mut self, lhs: impl ValueView, rhs: impl ValueView) -> PartialVMResult<()> {
        let eq_base = self.cost_table.instruction_gas_parameter.eq_base;
        let eq_per_abs_val_unit = self
            .cost_table
            .instruction_gas_parameter
            .eq_per_abs_val_unit;

        let lhs_abs_val_size = self
            .cost_table
            .abstract_value_parameter
            .abstract_value_size_dereferenced(lhs);
        let rhs_abs_val_size = self
            .cost_table
            .abstract_value_parameter
            .abstract_value_size_dereferenced(rhs);

        let cost = eq_base + eq_per_abs_val_unit * (lhs_abs_val_size + rhs_abs_val_size);

        self.charge_v1(cost)
    }

    fn charge_neq(&mut self, lhs: impl ValueView, rhs: impl ValueView) -> PartialVMResult<()> {
        let neq_base = self.cost_table.instruction_gas_parameter.eq_base;
        let neq_per_abs_val_unit = self
            .cost_table
            .instruction_gas_parameter
            .neq_per_abs_val_unit;

        let lhs_abs_val_size = self
            .cost_table
            .abstract_value_parameter
            .abstract_value_size_dereferenced(lhs);
        let rhs_abs_val_size = self
            .cost_table
            .abstract_value_parameter
            .abstract_value_size_dereferenced(rhs);

        let cost = neq_base + neq_per_abs_val_unit * (lhs_abs_val_size + rhs_abs_val_size);

        self.charge_v1(cost)
    }

    fn charge_borrow_global(
        &mut self,
        is_mut: bool,
        is_generic: bool,
        _ty: impl TypeView,
        _is_success: bool,
    ) -> PartialVMResult<()> {
        let imm_borrow_global_base = self
            .cost_table
            .instruction_gas_parameter
            .imm_borrow_global_base;
        let imm_borrow_global_generic_base = self
            .cost_table
            .instruction_gas_parameter
            .imm_borrow_global_generic_base;
        let mut_borrow_global_base = self
            .cost_table
            .instruction_gas_parameter
            .mut_borrow_global_base;
        let mut_borrow_global_generic_base = self
            .cost_table
            .instruction_gas_parameter
            .mut_borrow_global_generic_base;
        match (is_mut, is_generic) {
            (false, false) => self.charge_v1(imm_borrow_global_base),
            (false, true) => self.charge_v1(imm_borrow_global_generic_base),
            (true, false) => self.charge_v1(mut_borrow_global_base),
            (true, true) => self.charge_v1(mut_borrow_global_generic_base),
        }
    }

    fn charge_exists(
        &mut self,
        is_generic: bool,
        _ty: impl TypeView,
        _exists: bool,
    ) -> PartialVMResult<()> {
        let exists_base = self.cost_table.instruction_gas_parameter.exists_base;
        let exists_generic_base = self
            .cost_table
            .instruction_gas_parameter
            .exists_generic_base;

        match is_generic {
            false => self.charge_v1(exists_base),
            true => self.charge_v1(exists_generic_base),
        }
    }

    fn charge_move_from(
        &mut self,
        is_generic: bool,
        _ty: impl TypeView,
        _val: Option<impl ValueView>,
    ) -> PartialVMResult<()> {
        let move_from_base = self.cost_table.instruction_gas_parameter.move_from_base;
        let move_from_generic_base = self
            .cost_table
            .instruction_gas_parameter
            .move_from_generic_base;

        match is_generic {
            false => self.charge_v1(move_from_base),
            true => self.charge_v1(move_from_generic_base),
        }
    }

    fn charge_move_to(
        &mut self,
        is_generic: bool,
        _ty: impl TypeView,
        _val: impl ValueView,
        _is_success: bool,
    ) -> PartialVMResult<()> {
        let move_to_base = self.cost_table.instruction_gas_parameter.move_from_base;
        let move_to_generic_base = self
            .cost_table
            .instruction_gas_parameter
            .move_from_generic_base;

        match is_generic {
            false => self.charge_v1(move_to_base),
            true => self.charge_v1(move_to_generic_base),
        }
    }

    fn charge_vec_pack<'a>(
        &mut self,
        _ty: impl TypeView + 'a,
        args: impl ExactSizeIterator<Item = impl ValueView>,
    ) -> PartialVMResult<()> {
        let num_args = NumArgs::new(args.len() as u64);

        let vec_pack_base = self.cost_table.instruction_gas_parameter.vec_pack_base;
        let vec_pack_per_elem = self.cost_table.instruction_gas_parameter.vec_pack_per_elem;

        self.charge_v1(vec_pack_base + vec_pack_per_elem * num_args)
    }

    fn charge_vec_len(&mut self, _ty: impl TypeView) -> PartialVMResult<()> {
        self.charge_v1(self.cost_table.instruction_gas_parameter.vec_len_base)
    }

    fn charge_vec_borrow(
        &mut self,
        is_mut: bool,
        _ty: impl TypeView,
        _is_success: bool,
    ) -> PartialVMResult<()> {
        let vec_imm_borrow_base = self
            .cost_table
            .instruction_gas_parameter
            .vec_imm_borrow_base;
        let vec_mut_borrow_base = self
            .cost_table
            .instruction_gas_parameter
            .vec_mut_borrow_base;

        match is_mut {
            false => self.charge_v1(vec_imm_borrow_base),
            true => self.charge_v1(vec_mut_borrow_base),
        }
    }

    fn charge_vec_push_back(
        &mut self,
        _ty: impl TypeView,
        _val: impl ValueView,
    ) -> PartialVMResult<()> {
        self.charge_v1(self.cost_table.instruction_gas_parameter.vec_push_back_base)
    }

    fn charge_vec_pop_back(
        &mut self,
        _ty: impl TypeView,
        _val: Option<impl ValueView>,
    ) -> PartialVMResult<()> {
        self.charge_v1(self.cost_table.instruction_gas_parameter.vec_pop_back_base)
    }

    fn charge_vec_unpack(
        &mut self,
        _ty: impl TypeView,
        expect_num_elements: NumArgs,
        _elems: impl ExactSizeIterator<Item = impl ValueView>,
    ) -> PartialVMResult<()> {
        let vec_unpack_base = self.cost_table.instruction_gas_parameter.vec_unpack_base;
        let vec_unpack_per_expected_elem = self
            .cost_table
            .instruction_gas_parameter
            .vec_unpack_per_expected_elem;

        self.charge_v1(vec_unpack_base + vec_unpack_per_expected_elem * expect_num_elements)
    }

    fn charge_vec_swap(&mut self, _ty: impl TypeView) -> PartialVMResult<()> {
        self.charge_v1(self.cost_table.instruction_gas_parameter.vec_swap_base)
    }

    fn charge_load_resource(
        &mut self,
        _addr: AccountAddress,
        _ty: impl TypeView,
        _val: Option<impl ValueView>,
        _bytes_loaded: NumBytes,
    ) -> PartialVMResult<()> {
        // We don't have resource loading so don't need to account for it.
        Ok(())
    }

    fn charge_native_function(
        &mut self,
        amount: InternalGas,
        _ret_vals: Option<impl ExactSizeIterator<Item = impl ValueView>>,
    ) -> PartialVMResult<()> {
        self.charge_v1(amount)
    }

    fn charge_native_function_before_execution(
        &mut self,
        _ty_args: impl ExactSizeIterator<Item = impl TypeView>,
        _args: impl ExactSizeIterator<Item = impl ValueView>,
    ) -> PartialVMResult<()> {
        Ok(())
    }

    fn charge_drop_frame(
        &mut self,
        _locals: impl Iterator<Item = impl ValueView>,
    ) -> PartialVMResult<()> {
        Ok(())
    }
}

impl SwitchableGasMeter for MoveOSGasMeter {
    fn stop_metering(&mut self) {
        self.charge = false;
    }

    fn start_metering(&mut self) {
        self.charge = true;
    }

    fn is_metering(&self) -> bool {
        self.charge
    }
}

pub fn initial_instruction_parameter() -> InstructionParameter {
    InstructionParameter::initial()
}

pub fn initial_storage_parameter() -> StorageGasParameter {
    StorageGasParameter::initial()
}

pub fn initial_misc_parameter() -> AbstractValueSizeGasParameter {
    AbstractValueSizeGasParameter::initial()
}

pub fn from_on_chain_gas_schedule_to_instruction_parameter(
    gas_schedule: &BTreeMap<String, u64>,
) -> Option<InstructionParameter> {
    InstructionParameter::from_on_chain_gas_schedule(gas_schedule)
}

pub fn from_on_chain_gas_schedule_to_storage_parameter(
    gas_schedule: &BTreeMap<String, u64>,
) -> Option<StorageGasParameter> {
    StorageGasParameter::from_on_chain_gas_schedule(gas_schedule)
}

pub fn from_on_chain_gas_schedule_to_misc_parameter(
    gas_schedule: &BTreeMap<String, u64>,
) -> Option<AbstractValueSizeGasParameter> {
    AbstractValueSizeGasParameter::from_on_chain_gas_schedule(gas_schedule)
}

pub fn instruction_parameter_to_on_chain_gas_schedule(
    gas_parameter: InstructionParameter,
) -> Vec<(String, u64)> {
    gas_parameter.to_on_chain_gas_schedule()
}

pub fn storage_parameter_to_on_chain_gas_schedule(
    gas_parameter: StorageGasParameter,
) -> Vec<(String, u64)> {
    gas_parameter.to_on_chain_gas_schedule()
}

pub fn misc_parameter_to_on_chain_gas_schedule(
    gas_parameter: AbstractValueSizeGasParameter,
) -> Vec<(String, u64)> {
    gas_parameter.to_on_chain_gas_schedule()
}