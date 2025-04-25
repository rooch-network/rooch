// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use crate::log::{
    CallFrame, Dependency, ExecutionAndIOCosts, ExecutionGasEvent, FrameName, TransactionGasLog,
};
use move_binary_format::file_format::CodeOffset;
use move_binary_format::file_format_common::Opcodes;
use move_core_types::account_address::AccountAddress;
use move_core_types::gas_algebra::{InternalGas, NumArgs, NumBytes, NumTypeNodes};
use move_core_types::identifier::{IdentStr, Identifier};
use move_core_types::language_storage::{ModuleId, TypeTag};
use move_vm_types::gas::{GasMeter, SimpleInstruction};
use move_vm_types::natives::function::PartialVMResult;
use move_vm_types::views::{TypeView, ValueView};
use moveos_common::types::{ClassifiedGasMeter, GasStatement, SwitchableGasMeter};
use moveos_types::transaction::MoveAction;
use std::sync::{Arc, RwLock};

#[derive(Debug, Clone)]
pub struct GasProfiler<G> {
    base: G,
    frames: Arc<RwLock<Vec<CallFrame>>>,
    dependencies: Vec<Dependency>,
    metering: bool,
}

macro_rules! delegate_mut {
    ($(
        fn $fn: ident $(<$($lt: lifetime),*>)? (&mut self $(, $arg: ident : $ty: ty)* $(,)?) -> $ret_ty: ty;
    )*) => {
        $(fn $fn $(<$($lt)*>)? (&mut self, $($arg: $ty),*) -> $ret_ty {
            self.base.$fn($($arg),*)
        })*
    };
}

macro_rules! record_bytecode {
    ($(
        $([$op: expr])?
        fn $fn: ident $(<$($lt: lifetime),*>)? (&mut self $(, $arg: ident : $ty: ty)* $(,)?) -> PartialVMResult<()>;
    )*) => {
        $(fn $fn $(<$($lt)*>)? (&mut self, $($arg: $ty),*) -> PartialVMResult<()> {
            #[allow(unused)]
            use Opcodes::*;

            #[allow(unused)]
            let (cost, res) = self.delegate_charge(|base| base.$fn($($arg),*));

            $(
                self.record_bytecode($op, cost);
            )?

            res
        })*
    };
}

impl<G> GasProfiler<G> {
    pub fn new_function(
        base: G,
        module_id: ModuleId,
        func_name: Identifier,
        ty_args: Vec<TypeTag>,
    ) -> Self {
        Self {
            base,
            frames: Arc::new(RwLock::new(vec![CallFrame::new_function(
                module_id, func_name, ty_args,
            )])),
            dependencies: vec![],
            metering: true,
        }
    }
}

impl<G: GasMeter> GasProfiler<G> {
    fn record_gas_event(&mut self, event: ExecutionGasEvent) {
        if self.metering {
            self.frames
                .write()
                .unwrap()
                .last_mut()
                .unwrap()
                .events
                .push(event);
        }
    }

    fn record_bytecode(&mut self, op: Opcodes, cost: InternalGas) {
        if self.metering {
            self.record_gas_event(ExecutionGasEvent::Bytecode { op, cost })
        }
    }

    fn record_offset(&mut self, offset: CodeOffset) {
        if self.metering {
            self.record_gas_event(ExecutionGasEvent::Loc(offset))
        }
    }

    /// Delegate the charging call to the base gas meter and measure variation in balance.
    fn delegate_charge<F, R>(&mut self, charge: F) -> (InternalGas, R)
    where
        F: FnOnce(&mut G) -> R,
    {
        let old = self.base.balance_internal();
        let res = charge(&mut self.base);
        let new = self.base.balance_internal();
        let cost = old.checked_sub(new).expect("gas cost must be non-negative");

        (cost, res)
    }
}

impl<G: GasMeter> GasMeter for GasProfiler<G> {
    delegate_mut! {
        // Note: we only use this callback for memory tracking, not for charging gas.
        fn charge_ld_const_after_deserialization(&mut self, val: impl ValueView)
            -> PartialVMResult<()>;

        // Note: we don't use this to charge gas so no need to record anything.
        fn charge_native_function_before_execution(
            &mut self,
            ty_args: impl ExactSizeIterator<Item = impl TypeView> + Clone,
            args: impl ExactSizeIterator<Item = impl ValueView> + Clone,
        ) -> PartialVMResult<()>;

        // Note: we don't use this to charge gas so no need to record anything.
        fn charge_drop_frame(
            &mut self,
            locals: impl Iterator<Item = impl ValueView> + Clone,
        ) -> PartialVMResult<()>;
    }

    record_bytecode! {
        [POP]
        fn charge_pop(&mut self, popped_val: impl ValueView) -> PartialVMResult<()>;

        [LD_CONST]
        fn charge_ld_const(&mut self, size: NumBytes) -> PartialVMResult<()>;

        [COPY_LOC]
        fn charge_copy_loc(&mut self, val: impl ValueView) -> PartialVMResult<()>;

        [MOVE_LOC]
        fn charge_move_loc(&mut self, val: impl ValueView) -> PartialVMResult<()>;

        [ST_LOC]
        fn charge_store_loc(&mut self, val: impl ValueView) -> PartialVMResult<()>;

        [PACK]
        fn charge_pack(
            &mut self,
            is_generic: bool,
            args: impl ExactSizeIterator<Item = impl ValueView> + Clone,
        ) -> PartialVMResult<()>;

        [UNPACK]
        fn charge_unpack(
            &mut self,
            is_generic: bool,
            args: impl ExactSizeIterator<Item = impl ValueView> + Clone,
        ) -> PartialVMResult<()>;

        [READ_REF]
        fn charge_read_ref(&mut self, val: impl ValueView) -> PartialVMResult<()>;

        [WRITE_REF]
        fn charge_write_ref(
            &mut self,
            new_val: impl ValueView,
            old_val: impl ValueView,
        ) -> PartialVMResult<()>;

        [EQ]
        fn charge_eq(&mut self, lhs: impl ValueView, rhs: impl ValueView) -> PartialVMResult<()>;

        [NEQ]
        fn charge_neq(&mut self, lhs: impl ValueView, rhs: impl ValueView) -> PartialVMResult<()>;

        [
            match (is_mut, is_generic) {
                (false, false) => IMM_BORROW_GLOBAL,
                (false, true) => IMM_BORROW_GLOBAL_GENERIC,
                (true, false) => MUT_BORROW_GLOBAL,
                (true, true) => MUT_BORROW_GLOBAL_GENERIC
            }
        ]
        fn charge_borrow_global(
            &mut self,
            is_mut: bool,
            is_generic: bool,
            ty: impl TypeView,
            is_success: bool,
        ) -> PartialVMResult<()>;

        [if is_generic { EXISTS } else { EXISTS_GENERIC }]
        fn charge_exists(
            &mut self,
            is_generic: bool,
            ty: impl TypeView,
            exists: bool,
        ) -> PartialVMResult<()>;

        [if is_generic { MOVE_FROM } else { MOVE_FROM_GENERIC }]
        fn charge_move_from(
            &mut self,
            is_generic: bool,
            ty: impl TypeView,
            val: Option<impl ValueView>,
        ) -> PartialVMResult<()>;

        [if is_generic { MOVE_TO } else { MOVE_TO_GENERIC }]
        fn charge_move_to(
            &mut self,
            is_generic: bool,
            ty: impl TypeView,
            val: impl ValueView,
            is_success: bool,
        ) -> PartialVMResult<()>;

        [VEC_PACK]
        fn charge_vec_pack<'a>(
            &mut self,
            ty: impl TypeView + 'a,
            args: impl ExactSizeIterator<Item = impl ValueView> + Clone,
        ) -> PartialVMResult<()>;

        [VEC_LEN]
        fn charge_vec_len(&mut self, ty: impl TypeView) -> PartialVMResult<()>;

        [VEC_IMM_BORROW]
        fn charge_vec_borrow(
            &mut self,
            is_mut: bool,
            ty: impl TypeView,
            is_success: bool,
        ) -> PartialVMResult<()>;

        [VEC_PUSH_BACK]
        fn charge_vec_push_back(
            &mut self,
            ty: impl TypeView,
            val: impl ValueView,
        ) -> PartialVMResult<()>;

        [VEC_POP_BACK]
        fn charge_vec_pop_back(
            &mut self,
            ty: impl TypeView,
            val: Option<impl ValueView>,
        ) -> PartialVMResult<()>;

        [VEC_UNPACK]
        fn charge_vec_unpack(
            &mut self,
            ty: impl TypeView,
            expect_num_elements: NumArgs,
            elems: impl ExactSizeIterator<Item = impl ValueView> + Clone,
        ) -> PartialVMResult<()>;

        [VEC_SWAP]
        fn charge_vec_swap(&mut self, ty: impl TypeView) -> PartialVMResult<()>;
    }

    fn balance_internal(&self) -> InternalGas {
        self.base.balance_internal()
    }

    fn charge_simple_instr(&mut self, instr: SimpleInstruction) -> PartialVMResult<()> {
        let (cost, res) = self.delegate_charge(|base| base.charge_simple_instr(instr));

        self.record_bytecode(instr.to_opcode(), cost);

        // If we encounter a Ret instruction, it means the function has exited,
        // and we need to convert the current CallFrame into a GasEvent.
        // [call_frame_1, call_frame_2, call_frame_3]
        // [call_frame_1, call_frame_2(events: [Bytecode::Op, Call(call_frame_3)])]
        if matches!(instr, SimpleInstruction::Ret) && self.frames.read().unwrap().len() > 1 {
            let cur_frame = self
                .frames
                .write()
                .unwrap()
                .pop()
                .expect("frame must exist");
            let mut call_frames = self.frames.write().unwrap();
            let last_frame = call_frames.last_mut().expect("frame must exist");
            last_frame.events.push(ExecutionGasEvent::Call(cur_frame));
        }

        res
    }

    fn charge_br_true(&mut self, target_offset: Option<CodeOffset>) -> PartialVMResult<()> {
        let (cost, res) = self.delegate_charge(|base| base.charge_br_true(target_offset));

        self.record_bytecode(Opcodes::BR_TRUE, cost);
        if let Some(offset) = target_offset {
            self.record_offset(offset);
        }

        res
    }

    fn charge_br_false(&mut self, target_offset: Option<CodeOffset>) -> PartialVMResult<()> {
        let (cost, res) = self.delegate_charge(|base| base.charge_br_false(target_offset));

        self.record_bytecode(Opcodes::BR_FALSE, cost);
        if let Some(offset) = target_offset {
            self.record_offset(offset);
        }

        res
    }

    fn charge_branch(&mut self, target_offset: CodeOffset) -> PartialVMResult<()> {
        let (cost, res) = self.delegate_charge(|base| base.charge_branch(target_offset));

        self.record_bytecode(Opcodes::BRANCH, cost);
        self.record_offset(target_offset);

        res
    }

    fn charge_call(
        &mut self,
        module_id: &ModuleId,
        func_name: &str,
        args: impl ExactSizeIterator<Item = impl ValueView> + Clone,
        num_locals: NumArgs,
    ) -> PartialVMResult<()> {
        let (cost, res) =
            self.delegate_charge(|base| base.charge_call(module_id, func_name, args, num_locals));

        //println!("charge_call {:?}::{:?}", module_id, func_name);

        self.record_bytecode(Opcodes::CALL, cost);
        self.frames.write().unwrap().push(CallFrame::new_function(
            module_id.clone(),
            Identifier::new(func_name).unwrap(),
            vec![],
        ));

        res
    }

    fn charge_call_generic(
        &mut self,
        module_id: &ModuleId,
        func_name: &str,
        ty_args: impl ExactSizeIterator<Item = impl TypeView> + Clone,
        args: impl ExactSizeIterator<Item = impl ValueView> + Clone,
        num_locals: NumArgs,
    ) -> PartialVMResult<()> {
        let ty_tags = ty_args
            .clone()
            .map(|ty| ty.to_type_tag())
            .collect::<Vec<_>>();

        let (cost, res) = self.delegate_charge(|base| {
            base.charge_call_generic(module_id, func_name, ty_args, args, num_locals)
        });

        self.record_bytecode(Opcodes::CALL_GENERIC, cost);
        self.frames.write().unwrap().push(CallFrame::new_function(
            module_id.clone(),
            Identifier::new(func_name).unwrap(),
            ty_tags,
        ));

        res
    }

    fn charge_load_resource(
        &mut self,
        addr: AccountAddress,
        ty: impl TypeView,
        val: Option<impl ValueView>,
        bytes_loaded: NumBytes,
    ) -> PartialVMResult<()> {
        let ty_tag = ty.to_type_tag();

        let (cost, res) =
            self.delegate_charge(|base| base.charge_load_resource(addr, ty, val, bytes_loaded));

        self.record_gas_event(ExecutionGasEvent::LoadResource {
            addr,
            ty: ty_tag,
            cost,
        });

        res
    }

    fn charge_native_function(
        &mut self,
        amount: InternalGas,
        ret_vals: Option<impl ExactSizeIterator<Item = impl ValueView> + Clone>,
    ) -> PartialVMResult<()> {
        let (cost, res) =
            self.delegate_charge(|base| base.charge_native_function(amount, ret_vals));

        // Whenever a function gets called, the VM will notify the gas profiler
        // via `charge_call/charge_call_generic`.
        //
        // At this point of time, the gas profiler does not yet have an efficient way to determine
        // whether the function is a native or not, so it will blindly create a new frame.
        //
        // Later when it realizes the function is native, it will transform the original frame
        // into a native-specific event that does not contain recursive structures.
        let cur = self
            .frames
            .write()
            .unwrap()
            .pop()
            .expect("frame must exist");
        let (module_id, name, ty_args) = match cur.name {
            FrameName::Function {
                module_id,
                name,
                ty_args,
            } => (module_id, name, ty_args),
            FrameName::Script => unreachable!(),
        };
        // The following line of code is needed for correctness.
        //
        // This is because additional gas events may be produced after the frame has been
        // created and these events need to be preserved.
        self.frames
            .write()
            .unwrap()
            .last_mut()
            .unwrap()
            .events
            .extend(cur.events);

        self.record_gas_event(ExecutionGasEvent::CallNative {
            module_id,
            fn_name: name,
            ty_args,
            cost,
        });

        res
    }

    fn charge_create_ty(&mut self, num_nodes: NumTypeNodes) -> PartialVMResult<()> {
        let (cost, res) = self.delegate_charge(|base| base.charge_create_ty(num_nodes));

        self.record_gas_event(ExecutionGasEvent::CreateTy { cost });

        res
    }

    fn charge_dependency(
        &mut self,
        is_new: bool,
        addr: &AccountAddress,
        name: &IdentStr,
        size: NumBytes,
    ) -> PartialVMResult<()> {
        let (cost, res) =
            self.delegate_charge(|base| base.charge_dependency(is_new, addr, name, size));

        if !cost.is_zero() {
            self.dependencies.push(Dependency {
                is_new,
                id: ModuleId::new(*addr, name.to_owned()),
                size,
                cost,
            });
        }

        res
    }
}

pub trait ProfileGasMeter {
    fn finish(&mut self) -> TransactionGasLog;
}

impl<G: GasMeter> ProfileGasMeter for GasProfiler<G> {
    fn finish(&mut self) -> TransactionGasLog {
        while self.frames.read().unwrap().len() > 1 {
            let cur = self
                .frames
                .write()
                .unwrap()
                .pop()
                .expect("frame must exist");
            let mut call_frames = self.frames.write().unwrap();
            let last = call_frames.last_mut().expect("frame must exist");
            last.events.push(ExecutionGasEvent::Call(cur));
        }

        let exec_io = ExecutionAndIOCosts {
            total: self.base.balance_internal(),
            call_graph: self
                .frames
                .write()
                .unwrap()
                .pop()
                .expect("frame must exist"),
        };

        self.stop_metering();

        TransactionGasLog {
            exec_io,
            storage: InternalGas::zero(),
        }
    }
}

impl<G: GasMeter> ClassifiedGasMeter for GasProfiler<G> {
    fn charge_execution(&mut self, _gas_cost: u64) -> PartialVMResult<()> {
        Ok(())
    }

    fn charge_io_write(&mut self, _data_size: u64) -> PartialVMResult<()> {
        Ok(())
    }

    fn check_constrains(&self, _max_gas_amount: u64) -> PartialVMResult<()> {
        Ok(())
    }

    fn gas_statement(&self) -> GasStatement {
        GasStatement {
            execution_gas_used: InternalGas::zero(),
            storage_gas_used: InternalGas::zero(),
        }
    }
}

impl<G: GasMeter> SwitchableGasMeter for GasProfiler<G> {
    fn stop_metering(&mut self) {
        self.metering = false;
    }

    fn start_metering(&mut self) {
        self.metering = true;
    }

    fn is_metering(&self) -> bool {
        self.metering
    }
}

pub fn new_gas_profiler<G>(action: MoveAction, base_gas_meter: G) -> GasProfiler<G> {
    match action {
        MoveAction::Script(_) => unreachable!("Script payload is not supported yet"),
        MoveAction::Function(call) => GasProfiler::new_function(
            base_gas_meter,
            call.function_id.module_id,
            call.function_id.function_name,
            call.ty_args,
        ),
        MoveAction::ModuleBundle(_) => unreachable!("ModuleBundle payload is not supported yet"),
    }
}
