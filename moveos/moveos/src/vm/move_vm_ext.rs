// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use std::{borrow::Borrow, sync::Arc};

use super::{tx_argument_resolver::TxArgumentResolver, MoveResolverExt};
use move_binary_format::{
    compatibility::Compatibility,
    errors::{PartialVMError, VMResult},
    file_format::AbilitySet,
};
use move_bytecode_verifier::VerifierConfig;
use move_core_types::{
    account_address::AccountAddress,
    effects::{ChangeSet, Event},
    identifier::IdentStr,
    language_storage::{ModuleId, TypeTag},
    value::MoveTypeLayout,
};
use moveos_stdlib::natives::moveos_stdlib::raw_table::NativeTableContext;
use move_vm_runtime::{
    config::VMConfig,
    move_vm::MoveVM,
    native_extensions::NativeContextExtensions,
    session::{LoadedFunctionInstantiation, SerializedReturnValues, Session},
};
use move_vm_types::{
    data_store::DataStore,
    gas::GasMeter,
    loaded_data::runtime_types::{CachedStructIndex, StructType, Type},
};
use moveos_stdlib::natives::{
    self, GasParameters,
};
use moveos_types::tx_context::TxContext;

pub struct MoveVmExt {
    inner: MoveVM,
}

impl MoveVmExt {
    pub fn new() -> VMResult<Self> {
        let gas_params = GasParameters::zeros();
        Ok(Self {
            inner: MoveVM::new_with_config(
                natives::all_natives(gas_params),
                VMConfig {
                    verifier: VerifierConfig::default(),
                    max_binary_format_version: 6,
                    paranoid_type_checks: false,
                },
            )?,
        })
    }

    pub fn new_session<'r, S: MoveResolverExt>(
        &self,
        remote: &'r S,
    ) -> SessionExt<'r, '_, S> {
        let mut extensions = NativeContextExtensions::default();
        //let txn_hash: [u8; 32] = session_id.into();

        extensions.add(NativeTableContext::new(remote));
        //extensions.add(NativeObjectContext::new(remote));

        // The VM code loader has bugs around module upgrade. After a module upgrade, the internal
        // cache needs to be flushed to work around those bugs.
        self.inner.flush_loader_cache_if_invalidated();

        let session = self.inner.new_session_with_extensions(remote, extensions);
        SessionExt::new(session)
    }
}

pub struct SessionExt<'r, 'l, S> {
    session: Session<'r, 'l, S>,
}

impl<'r, 'l, S> SessionExt<'r, 'l, S>
where
    S: MoveResolverExt,
{
    pub fn new(session: Session<'r, 'l, S>) -> Self {
        Self { session }
    }

    pub fn resolve_args(
        &self,
        tx_context: &TxContext,
        func: LoadedFunctionInstantiation,
        args: Vec<Vec<u8>>,
    ) -> Result<Vec<Vec<u8>>, PartialVMError> {
        tx_context.resolve_argument(self, &func, args)
        // let object_context = self
        //     .session
        //     .get_native_extensions()
        //     .get::<NativeObjectContext>();
        // object_context.resolve_argument(self, &func, args)
    }

    /************** Proxy function */

    pub fn execute_entry_function(
        &mut self,
        module: &ModuleId,
        function_name: &IdentStr,
        ty_args: Vec<TypeTag>,
        args: Vec<impl Borrow<[u8]>>,
        gas_meter: &mut impl GasMeter,
    ) -> VMResult<SerializedReturnValues> {
        self.session
            .execute_entry_function(module, function_name, ty_args, args, gas_meter)
    }

    pub fn execute_function_bypass_visibility(
        &mut self,
        module: &ModuleId,
        function_name: &IdentStr,
        ty_args: Vec<TypeTag>,
        args: Vec<impl Borrow<[u8]>>,
        gas_meter: &mut impl GasMeter,
    ) -> VMResult<SerializedReturnValues> {
        self.session.execute_function_bypass_visibility(
            module,
            function_name,
            ty_args,
            args,
            gas_meter,
        )
    }

    pub fn execute_script(
        &mut self,
        script: impl Borrow<[u8]>,
        ty_args: Vec<TypeTag>,
        args: Vec<impl Borrow<[u8]>>,
        gas_meter: &mut impl GasMeter,
    ) -> VMResult<SerializedReturnValues> {
        self.session
            .execute_script(script, ty_args, args, gas_meter)
    }

    pub fn publish_module(
        &mut self,
        module: Vec<u8>,
        sender: AccountAddress,
        gas_meter: &mut impl GasMeter,
    ) -> VMResult<()> {
        self.session.publish_module(module, sender, gas_meter)
    }

    pub fn publish_module_bundle(
        &mut self,
        modules: Vec<Vec<u8>>,
        sender: AccountAddress,
        gas_meter: &mut impl GasMeter,
    ) -> VMResult<()> {
        self.session
            .publish_module_bundle(modules, sender, gas_meter)
    }

    pub fn publish_module_bundle_with_compat_config(
        &mut self,
        modules: Vec<Vec<u8>>,
        sender: AccountAddress,
        gas_meter: &mut impl GasMeter,
        compat_config: Compatibility,
    ) -> VMResult<()> {
        self.session.publish_module_bundle_with_compat_config(
            modules,
            sender,
            gas_meter,
            compat_config,
        )
    }

    pub fn num_mutated_accounts(&self, sender: &AccountAddress) -> u64 {
        self.session.num_mutated_accounts(sender)
    }

    pub fn finish(self) -> VMResult<(ChangeSet, Vec<Event>)> {
        self.session.finish()
    }

    pub fn finish_with_extensions(
        self,
    ) -> VMResult<(ChangeSet, Vec<Event>, NativeContextExtensions<'r>)> {
        self.session.finish_with_extensions()
    }

    /// Load a script and all of its types into cache
    pub fn load_script(
        &self,
        script: impl Borrow<[u8]>,
        ty_args: Vec<TypeTag>,
    ) -> VMResult<LoadedFunctionInstantiation> {
        self.session.load_script(script, ty_args)
    }

    /// Load a module, a function, and all of its types into cache
    pub fn load_function(
        &self,
        module_id: &ModuleId,
        function_name: &IdentStr,
        type_arguments: &[TypeTag],
    ) -> VMResult<LoadedFunctionInstantiation> {
        self.session
            .load_function(module_id, function_name, type_arguments)
    }

    pub fn load_type(&self, type_tag: &TypeTag) -> VMResult<Type> {
        self.session.load_type(type_tag)
    }

    pub fn get_type_layout(&self, type_tag: &TypeTag) -> VMResult<MoveTypeLayout> {
        self.session.get_type_layout(type_tag)
    }

    pub fn get_fully_annotated_type_layout(&self, type_tag: &TypeTag) -> VMResult<MoveTypeLayout> {
        self.session.get_fully_annotated_type_layout(type_tag)
    }

    pub fn get_type_tag(&self, ty: &Type) -> VMResult<TypeTag> {
        self.session.get_type_tag(ty)
    }

    pub fn get_struct_type(&self, index: CachedStructIndex) -> Option<Arc<StructType>> {
        self.session.get_struct_type(index)
    }

    pub fn get_type_abilities(&self, ty: &Type) -> VMResult<AbilitySet> {
        self.session.get_type_abilities(ty)
    }

    pub fn get_data_store(&mut self) -> &mut dyn DataStore {
        self.session.get_data_store()
    }

    pub fn get_native_extensions(&self) -> &NativeContextExtensions<'r> {
        self.session.get_native_extensions()
    }
}

impl AsRef<MoveVM> for MoveVmExt {
    fn as_ref(&self) -> &MoveVM {
        &self.inner
    }
}
