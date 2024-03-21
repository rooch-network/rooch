// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use move_binary_format::file_format::{
    Bytecode, ConstantPoolIndex, FieldHandleIndex, FieldInstantiationIndex, FunctionHandleIndex,
    FunctionInstantiationIndex, SignatureIndex, StructDefInstantiationIndex, StructDefinitionIndex,
};
use move_binary_format::file_format_common::instruction_key;
use move_vm_test_utils::gas_schedule::GasCost;
use once_cell::sync::Lazy;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

#[allow(non_camel_case_types)]
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq, PartialOrd, Ord)]
#[repr(u8)]
pub enum N {
    SHA2_256 = 0,
    SHA3_256 = 1,
    ED25519_VERIFY = 2,
    ED25519_THRESHOLD_VERIFY = 3,
    BCS_TO_BYTES = 4,
    LENGTH = 5,
    EMPTY = 6,
    BORROW = 7,
    BORROW_MUT = 8,
    PUSH_BACK = 9,
    POP_BACK = 10,
    DESTROY_EMPTY = 11,
    SWAP = 12,
    ED25519_VALIDATE_KEY = 13,
    SIGNER_BORROW = 14,
    CREATE_SIGNER = 15,
    DESTROY_SIGNER = 16,
    EMIT_EVENT = 17,
    BCS_TO_ADDRESS = 18,
    TOKEN_NAME_OF = 19,
    KECCAK_256 = 20,
    RIPEMD160 = 21,
    ECRECOVER = 22,
    U256_FROM_BYTES = 23,
    U256_ADD = 24,
    U256_SUB = 25,
    U256_MUL = 26,
    U256_DIV = 27,
    U256_REM = 28,
    U256_POW = 29,
    VEC_APPEND = 30,
    VEC_REMOVE = 31,
    VEC_REVERSE = 32,
    TABLE_NEW = 33,
    TABLE_INSERT = 34,
    TABLE_BORROW = 35,
    TABLE_REMOVE = 36,
    TABLE_CONTAINS = 37,
    TABLE_DESTROY = 38,
    TABLE_DROP = 39,
    STRING_CHECK_UT8 = 40,
    STRING_SUB_STR = 41,
    SRING_CHAR_BOUNDARY = 42,
    STRING_INDEX_OF = 43,
    FROMBCS_FROM_BYTES = 44,
    SECP256K1_ECDSA_RECOVER_INTERNAL = 45,
    VECTOR_SPAWN_FROM = 46,
}

impl N {
    //note: should change this value when add new native function.
    pub const NUMBER_OF_NATIVE_FUNCTIONS: usize = 47;
}

// TODO: Place the gas_schedule on the blockchain instead of hardcoding it in the code.
pub fn v5_native_table() -> Vec<GasCost> {
    let mut raw_native_table = vec![
        (N::SHA2_256, GasCost::new(21, 1)),
        (N::SHA3_256, GasCost::new(64, 1)),
        (N::ED25519_VERIFY, GasCost::new(61, 1)),
        (N::ED25519_THRESHOLD_VERIFY, GasCost::new(3351, 1)),
        (N::BCS_TO_BYTES, GasCost::new(181, 1)),
        (N::LENGTH, GasCost::new(98, 1)),
        (N::EMPTY, GasCost::new(84, 1)),
        (N::BORROW, GasCost::new(1334, 1)),
        (N::BORROW_MUT, GasCost::new(1902, 1)),
        (N::PUSH_BACK, GasCost::new(53, 1)),
        (N::POP_BACK, GasCost::new(227, 1)),
        (N::DESTROY_EMPTY, GasCost::new(572, 1)),
        (N::SWAP, GasCost::new(1436, 1)),
        (N::ED25519_VALIDATE_KEY, GasCost::new(26, 1)),
        (N::SIGNER_BORROW, GasCost::new(353, 1)),
        (N::CREATE_SIGNER, GasCost::new(24, 1)),
        (N::DESTROY_SIGNER, GasCost::new(212, 1)),
        (N::EMIT_EVENT, GasCost::new(52, 1)),
        (N::BCS_TO_ADDRESS, GasCost::new(26, 1)),
        (N::TOKEN_NAME_OF, GasCost::new(2002, 1)),
        (N::KECCAK_256, GasCost::new(64, 1)),
        (N::RIPEMD160, GasCost::new(64, 1)),
        (N::ECRECOVER, GasCost::new(128, 1)),
        (N::U256_FROM_BYTES, GasCost::new(2, 1)),
        (N::U256_ADD, GasCost::new(4, 1)),
        (N::U256_SUB, GasCost::new(4, 1)),
        (N::U256_MUL, GasCost::new(4, 1)),
        (N::U256_DIV, GasCost::new(10, 1)),
        (N::U256_REM, GasCost::new(4, 1)),
        (N::U256_POW, GasCost::new(8, 1)),
        (N::VEC_APPEND, GasCost::new(40, 1)),
        (N::VEC_REMOVE, GasCost::new(20, 1)),
        (N::VEC_REVERSE, GasCost::new(10, 1)),
        (N::TABLE_NEW, GasCost::new(4, 1)),
        (N::TABLE_INSERT, GasCost::new(4, 1)),
        (N::TABLE_BORROW, GasCost::new(10, 1)),
        (N::TABLE_REMOVE, GasCost::new(8, 1)),
        (N::TABLE_CONTAINS, GasCost::new(40, 1)),
        (N::TABLE_DESTROY, GasCost::new(20, 1)),
        (N::TABLE_DROP, GasCost::new(73, 1)),
        (N::STRING_CHECK_UT8, GasCost::new(4, 1)),
        (N::STRING_SUB_STR, GasCost::new(4, 1)),
        (N::SRING_CHAR_BOUNDARY, GasCost::new(4, 1)),
        (N::STRING_INDEX_OF, GasCost::new(4, 1)),
        (N::FROMBCS_FROM_BYTES, GasCost::new(4, 1)),
        (N::SECP256K1_ECDSA_RECOVER_INTERNAL, GasCost::new(4, 1)),
        (N::VECTOR_SPAWN_FROM, GasCost::new(4, 1)),
    ];
    raw_native_table.sort_by_key(|cost| cost.0 as u64);
    raw_native_table
        .into_iter()
        .map(|(_, cost)| cost)
        .collect::<Vec<_>>()
}

pub static G_LATEST_NATIVE_TABLE: Lazy<Vec<GasCost>> = Lazy::new(|| {
    let native_table = v5_native_table();

    debug_assert!(
        native_table.len() == N::NUMBER_OF_NATIVE_FUNCTIONS,
        "all native functions must be in the cost table"
    );
    native_table
});

#[derive(Clone, Debug, Serialize, PartialEq, Eq, Deserialize)]
pub struct GasConstants {
    /// The cost per-byte read from global storage.
    pub global_memory_per_byte_cost: u64,

    /// The cost per-byte written to storage.
    pub global_memory_per_byte_write_cost: u64,

    /// The flat minimum amount of gas required for any transaction.
    /// Charged at the start of execution.
    pub min_transaction_gas_units: u64,

    /// Any transaction over this size will be charged an additional amount per byte.
    pub large_transaction_cutoff: u64,

    /// The units of gas that to be charged per byte over the `large_transaction_cutoff` in addition to
    /// `min_transaction_gas_units` for transactions whose size exceeds `large_transaction_cutoff`.
    pub intrinsic_gas_per_byte: u64,

    /// ~5 microseconds should equal one unit of computational gas. We bound the maximum
    /// computational time of any given transaction at roughly 20 seconds. We want this number and
    /// `MAX_PRICE_PER_GAS_UNIT` to always satisfy the inequality that
    /// MAXIMUM_NUMBER_OF_GAS_UNITS * MAX_PRICE_PER_GAS_UNIT < min(u64::MAX, GasUnits<GasCarrier>::MAX)
    /// NB: The bound is set quite high since custom scripts aren't allowed except from predefined
    /// and vetted senders.
    pub maximum_number_of_gas_units: u64,

    /// The minimum gas price that a transaction can be submitted with.
    pub min_price_per_gas_unit: u64,

    /// The maximum gas unit price that a transaction can be submitted with.
    pub max_price_per_gas_unit: u64,

    pub max_transaction_size_in_bytes: u64,

    pub gas_unit_scaling_factor: u64,
    pub default_account_size: u64,
}

/// Any transaction over this size will be charged `INTRINSIC_GAS_PER_BYTE` per byte
pub static G_LARGE_TRANSACTION_CUTOFF: u64 = 600;

pub static G_MAX_TRANSACTION_SIZE_IN_BYTES_V1: u64 = 4096 * 10;
pub static G_MAX_TRANSACTION_SIZE_IN_BYTES_V3: u64 = 128 * 1024;
pub static G_DEFAULT_ACCOUNT_SIZE: u64 = 800;

pub static G_GAS_CONSTANTS: Lazy<GasConstants> = Lazy::new(|| {
    GasConstants {
        global_memory_per_byte_cost: 4,
        global_memory_per_byte_write_cost: 9,
        min_transaction_gas_units: 600,
        large_transaction_cutoff: G_LARGE_TRANSACTION_CUTOFF,
        intrinsic_gas_per_byte: 8,
        maximum_number_of_gas_units: 40_000_000, //must less than base_block_gas_limit
        min_price_per_gas_unit: 1,
        max_price_per_gas_unit: 10_000,
        max_transaction_size_in_bytes: G_MAX_TRANSACTION_SIZE_IN_BYTES_V3,
        gas_unit_scaling_factor: 1,
        default_account_size: G_DEFAULT_ACCOUNT_SIZE,
    }
});

pub static G_LATEST_GAS_CONSTANTS: Lazy<GasConstants> = Lazy::new(|| G_GAS_CONSTANTS.clone());

#[derive(Clone, Debug, Serialize, PartialEq, Eq, Deserialize)]
pub struct CostTable {
    pub instruction_table: Vec<GasCost>,
    pub native_table: Vec<GasCost>,
    pub gas_constants: GasConstants,
}

pub fn instruction_table() -> Vec<GasCost> {
    use Bytecode::*;
    let mut instrs = vec![
        (MoveTo(StructDefinitionIndex::new(0)), GasCost::new(13, 1)),
        (
            MoveToGeneric(StructDefInstantiationIndex::new(0)),
            GasCost::new(27, 1),
        ),
        (
            MoveFrom(StructDefinitionIndex::new(0)),
            GasCost::new(459, 1),
        ),
        (
            MoveFromGeneric(StructDefInstantiationIndex::new(0)),
            GasCost::new(13, 1),
        ),
        (BrTrue(0), GasCost::new(1, 1)),
        (WriteRef, GasCost::new(1, 1)),
        (Mul, GasCost::new(1, 1)),
        (MoveLoc(0), GasCost::new(1, 1)),
        (And, GasCost::new(1, 1)),
        (Pop, GasCost::new(1, 1)),
        (BitAnd, GasCost::new(2, 1)),
        (ReadRef, GasCost::new(1, 1)),
        (Sub, GasCost::new(1, 1)),
        (MutBorrowField(FieldHandleIndex::new(0)), GasCost::new(1, 1)),
        (
            MutBorrowFieldGeneric(FieldInstantiationIndex::new(0)),
            GasCost::new(1, 1),
        ),
        (ImmBorrowField(FieldHandleIndex::new(0)), GasCost::new(1, 1)),
        (
            ImmBorrowFieldGeneric(FieldInstantiationIndex::new(0)),
            GasCost::new(1, 1),
        ),
        (Add, GasCost::new(1, 1)),
        (CopyLoc(0), GasCost::new(1, 1)),
        (StLoc(0), GasCost::new(1, 1)),
        (Ret, GasCost::new(638, 1)),
        (Lt, GasCost::new(1, 1)),
        (LdU8(0), GasCost::new(1, 1)),
        (LdU64(0), GasCost::new(1, 1)),
        (LdU128(0), GasCost::new(1, 1)),
        (CastU8, GasCost::new(2, 1)),
        (CastU64, GasCost::new(1, 1)),
        (CastU128, GasCost::new(1, 1)),
        (Abort, GasCost::new(1, 1)),
        (MutBorrowLoc(0), GasCost::new(2, 1)),
        (ImmBorrowLoc(0), GasCost::new(1, 1)),
        (LdConst(ConstantPoolIndex::new(0)), GasCost::new(1, 1)),
        (Ge, GasCost::new(1, 1)),
        (Xor, GasCost::new(1, 1)),
        (Shl, GasCost::new(2, 1)),
        (Shr, GasCost::new(1, 1)),
        (Neq, GasCost::new(1, 1)),
        (Not, GasCost::new(1, 1)),
        (Call(FunctionHandleIndex::new(0)), GasCost::new(1132, 1)),
        (
            CallGeneric(FunctionInstantiationIndex::new(0)),
            GasCost::new(582, 1),
        ),
        (Le, GasCost::new(2, 1)),
        (Branch(0), GasCost::new(1, 1)),
        (Unpack(StructDefinitionIndex::new(0)), GasCost::new(2, 1)),
        (
            UnpackGeneric(StructDefInstantiationIndex::new(0)),
            GasCost::new(2, 1),
        ),
        (Or, GasCost::new(2, 1)),
        (LdFalse, GasCost::new(1, 1)),
        (LdTrue, GasCost::new(1, 1)),
        (Mod, GasCost::new(1, 1)),
        (BrFalse(0), GasCost::new(1, 1)),
        (Exists(StructDefinitionIndex::new(0)), GasCost::new(41, 1)),
        (
            ExistsGeneric(StructDefInstantiationIndex::new(0)),
            GasCost::new(34, 1),
        ),
        (BitOr, GasCost::new(2, 1)),
        (FreezeRef, GasCost::new(1, 1)),
        (
            MutBorrowGlobal(StructDefinitionIndex::new(0)),
            GasCost::new(21, 1),
        ),
        (
            MutBorrowGlobalGeneric(StructDefInstantiationIndex::new(0)),
            GasCost::new(15, 1),
        ),
        (
            ImmBorrowGlobal(StructDefinitionIndex::new(0)),
            GasCost::new(23, 1),
        ),
        (
            ImmBorrowGlobalGeneric(StructDefInstantiationIndex::new(0)),
            GasCost::new(14, 1),
        ),
        (Div, GasCost::new(3, 1)),
        (Eq, GasCost::new(1, 1)),
        (Gt, GasCost::new(1, 1)),
        (Pack(StructDefinitionIndex::new(0)), GasCost::new(2, 1)),
        (
            PackGeneric(StructDefInstantiationIndex::new(0)),
            GasCost::new(2, 1),
        ),
        (Nop, GasCost::new(1, 1)),
        (VecPack(SignatureIndex::new(0), 0), GasCost::new(84, 1)),
        (VecLen(SignatureIndex::new(0)), GasCost::new(98, 1)),
        (VecImmBorrow(SignatureIndex::new(0)), GasCost::new(1334, 1)),
        (VecMutBorrow(SignatureIndex::new(0)), GasCost::new(1902, 1)),
        (VecPushBack(SignatureIndex::new(0)), GasCost::new(53, 1)),
        (VecPopBack(SignatureIndex::new(0)), GasCost::new(227, 1)),
        (VecUnpack(SignatureIndex::new(0), 0), GasCost::new(572, 1)),
        (VecSwap(SignatureIndex::new(0)), GasCost::new(1436, 1)),
    ];
    // Note that the DiemVM is expecting the table sorted by instruction order.
    instrs.sort_by_key(|cost| instruction_key(&cost.0));
    instrs.into_iter().map(|(_, cost)| cost).collect::<Vec<_>>()
}

pub static G_LATEST_INSTRUCTION_TABLE: Lazy<Vec<GasCost>> = Lazy::new(instruction_table);

pub fn latest_cost_table(gas_constants: GasConstants) -> CostTable {
    CostTable {
        instruction_table: G_LATEST_INSTRUCTION_TABLE.clone(),
        native_table: G_LATEST_NATIVE_TABLE.clone(),
        gas_constants,
    }
}

pub static G_LATEST_GAS_COST_TABLE: Lazy<CostTable> =
    Lazy::new(|| latest_cost_table(G_LATEST_GAS_CONSTANTS.clone()));

pub struct GasSchedule {
    pub entries: Vec<(String, u64)>,
}

impl GasSchedule {
    pub fn to_btree_map(self) -> BTreeMap<String, u64> {
        // TODO: what if the gas schedule contains duplicated entries?
        self.entries.into_iter().collect()
    }

    #[cfg(feature = "print_gas_info")]
    pub fn info(&self, message: &str) {
        let mut gas_info = String::from("GasSchedule info begin\n");
        gas_info.push_str(&format!("{}\n", message));
        self.entries.iter().for_each(|(key, value)| {
            gas_info.push_str(&format!("key = {}, gas value = {}\n", key, value));
        });
        gas_info.push_str("GasSchedule info end\n");
        info!("{}", gas_info);
    }

    /// check if there is any one of entry different from the other
    /// if it is, return true otherwise false
    pub fn is_different(&self, other: &GasSchedule) -> bool {
        let diff_len = self.entries.len() != other.entries.len();
        if diff_len {
            debug_assert!(
                !diff_len,
                "self.entries.len() = {} not the same as other.entries.len() = {}",
                self.entries.len(),
                other.entries.len()
            );
            return true;
        }
        self.entries
            .iter()
            .enumerate()
            .any(|(index, (key, value))| {
                let tuple = &other.entries[index];
                let diff = &tuple.0 != key || &tuple.1 != value;
                debug_assert!(
                    !diff,
                    "self.entries[{}] = {} not the same as other.entries[{}] = {}",
                    key, value, tuple.0, tuple.1
                );
                diff
            })
    }
}

static G_INSTR_STRS: Lazy<Vec<&str>> = Lazy::new(|| {
    vec![
        "instr.pop",
        "instr.ret",
        "instr.br_true",
        "instr.br_false",
        "instr.branch",
        "instr.ld_u64",
        "instr.ld_const.per_byte",
        "instr.ld_true",
        "instr.ld_false",
        "instr.copy_loc.per_abs_mem_unit",
        "instr.move_loc.per_abs_mem_unit",
        "instr.st_loc.per_abs_mem_unit",
        "instr.mut_borrow_loc",
        "instr.imm_borrow_loc",
        "instr.mut_borrow_field",
        "instr.imm_borrow_field",
        "instr.call.per_arg",
        "instr.pack.per_abs_mem_unit",
        "instr.unpack.per_abs_mem_unit",
        "instr.read_ref.per_abs_mem_unit",
        "instr.write_ref.per_abs_mem_unit",
        "instr.add",
        "instr.sub",
        "instr.mul",
        "instr.mod",
        "instr.div",
        "instr.bit_or",
        "instr.bit_and",
        "instr.xor",
        "instr.or",
        "instr.and",
        "instr.not",
        "instr.eq.per_abs_mem_unit",
        "instr.neq.per_abs_mem_unit",
        "instr.lt",
        "instr.gt",
        "instr.le",
        "instr.ge",
        "instr.abort",
        "instr.nop",
        "instr.exists.per_abs_mem_unit",
        "instr.mut_borrow_global.per_abs_mem_unit",
        "instr.imm_borrow_global.per_abs_mem_unit",
        "instr.move_from.per_abs_mem_unit",
        "instr.move_to.per_abs_mem_unit",
        "instr.freeze_ref",
        "instr.shl",
        "instr.shr",
        "instr.ld_u8",
        "instr.ld_u128",
        "instr.cast_u8",
        "instr.cast_u64",
        "instr.cast_u128",
        "instr.mut_borrow_field_generic.base",
        "instr.imm_borrow_field_generic.base",
        "instr.call_generic.per_arg",
        "instr.pack_generic.per_abs_mem_unit",
        "instr.unpack_generic.per_abs_mem_unit",
        "instr.exists_generic.per_abs_mem_unit",
        "instr.mut_borrow_global_generic.per_abs_mem_unit",
        "instr.imm_borrow_global_generic.per_abs_mem_unit",
        "instr.move_from_generic.per_abs_mem_unit",
        "instr.move_to_generic.per_abs_mem_unit",
        "instr.vec_pack.per_elem",
        "instr.vec_len.base",
        "instr.vec_imm_borrow.base",
        "instr.vec_mut_borrow.base",
        "instr.vec_push_back.per_abs_mem_unit",
        "instr.vec_pop_back.base",
        "instr.vec_unpack.per_expected_elem",
        "instr.vec_swap.base",
    ]
});

// TODO: Place the gas_schedule on the blockchain instead of hardcoding it in the code.
static G_NATIVE_STRS: Lazy<Vec<&str>> = Lazy::new(|| {
    vec![
        "move_stdlib.hash.sha2_256.per_byte",
        "move_stdlib.hash.sha3_256.per_byte",
        // ED25519_THRESHOLD_VERIFY 3 this native funciton is deprecated, ignore, use ""
        "",
        "move_stdlib.bcs.to_bytes.per_byte_serialized",
        "move_stdlib.vector.length.base",
        "move_stdlib.vector.empty.base",
        "move_stdlib.vector.borrow.base",
        // Vector::borrow_mut is same Vector::borrow ignore ""
        "",
        "move_stdlib.vector.push_back.legacy_per_abstract_memory_unit",
        "move_stdlib.vector.pop_back.base",
        "move_stdlib.vector.destroy_empty.base",
        "move_stdlib.vector.swap.base",
        "move_stdlib.signer.borrow_address.base",
        "nursery.event.write_to_event_store.unit_cost",
        "move_stdlib.bcs.to_address.per_byte",
        "move_stdlib.vector.append.legacy_per_abstract_memory_unit",
        "move_stdlib.vector.remove.legacy_per_abstract_memory_unit",
        "move_stdlib.vector.reverse.legacy_per_abstract_memory_unit",
        "table.new_table_handle.base",
        "table.add_box.per_byte_serialized",
        "table.borrow_box.per_byte_serialized",
        "table.remove_box.per_byte_serialized",
        "table.contains_box.per_byte_serialized",
        "table.contains_box_with_value_type.per_byte_serialized",
        "table.destroy_empty_box.base",
        "table.drop_unchecked_box.base",
        "table.box_length.base",
        "move_stdlib.string.check_utf8.per_byte",
        "move_stdlib.string.sub_string.per_byte",
        "move_stdlib.string.is_char_boundary.base",
        "move_stdlib.string.index_of.per_byte_searched",
        "move_stdlib.vector.spawn_from.legacy_per_abstract_memory_unit",
    ]
});

impl From<&CostTable> for GasSchedule {
    fn from(cost_table: &CostTable) -> Self {
        let mut entries = vec![];

        let instrs = cost_table.instruction_table.clone();
        for (idx, cost) in instrs.into_iter().enumerate() {
            entries.push((G_INSTR_STRS[idx].to_string(), cost.total()));
        }
        entries.push(("instr.ld_u16".to_string(), 3));
        entries.push(("instr.ld_u32".to_string(), 2));
        entries.push(("instr.ld_u256".to_string(), 3));
        entries.push(("instr.cast_u16".to_string(), 3));
        entries.push(("instr.cast_u32".to_string(), 2));
        entries.push(("instr.cast_u256".to_string(), 3));

        let natives = cost_table.native_table.clone();
        for (idx, cost) in natives.into_iter().enumerate() {
            if G_NATIVE_STRS[idx].is_empty() {
                continue;
            }
            let (a, b) = (G_NATIVE_STRS[idx].to_string(), cost.total());
            println!("idx {:}, native name {:?}, cost {:?}", idx, a, b);
            entries.push((G_NATIVE_STRS[idx].to_string(), cost.total()));
        }

        // native_table don't have these
        entries.push(("nursery.debug.print.base_cost".to_string(), 1));
        entries.push(("nursery.debug.print_stack_trace.base_cost".to_string(), 1));

        entries.push((
            "move_stdlib.hash.sha2_256.legacy_min_input_len".to_string(),
            1,
        ));
        entries.push((
            "move_stdlib.hash.sha3_256.legacy_min_input_len".to_string(),
            1,
        ));
        entries.push(("move_stdlib.bcs.to_bytes.failure".to_string(), 182));
        entries.push((
            "move_stdlib.bcs.to_bytes.legacy_min_output_size".to_string(),
            1,
        ));

        Self { entries }
    }
}

pub fn get_global_gas_schedule() -> GasSchedule {
    GasSchedule::from(&G_LATEST_GAS_COST_TABLE.clone())
}
