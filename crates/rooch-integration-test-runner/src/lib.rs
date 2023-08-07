// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use clap::Parser;
use codespan_reporting::diagnostic::Severity;
use codespan_reporting::term::termcolor::Buffer;
use move_command_line_common::address::NumericalAddress;
use move_command_line_common::files::verify_and_create_named_address_mapping;
use move_command_line_common::{address::ParsedAddress, values::ParsableValue};
use move_compiler::compiled_unit::CompiledUnitEnum;
use move_compiler::FullyCompiledProgram;
use move_core_types::effects::{ChangeSet, Op};
use move_transactional_test_runner::{
    tasks::{InitCommand, SyntaxChoice},
    vm_test_harness::view_resource_in_move_storage,
};
use move_vm_runtime::session::SerializedReturnValues;
use moveos::moveos::MoveOS;
use moveos::moveos_test_runner::MoveOSTestAdapter;
use moveos::moveos_test_runner::{CompiledState, TaskInput};
use moveos_store::MoveOSStore;
use moveos_types::move_types::FunctionId;
use moveos_types::object::ObjectID;
use moveos_types::state::StateChangeSet;
use moveos_types::state_resolver::AnnotatedStateReader;
use moveos_types::transaction::{MoveAction, MoveOSTransaction, TransactionOutput};
use moveos_verifier::build::build_model;
use moveos_verifier::metadata::run_extended_checks;
use regex::Regex;
use rooch_genesis::RoochGenesis;
use std::path::PathBuf;
use std::{collections::BTreeMap, path::Path};

pub struct MoveOSTestRunner<'a> {
    compiled_state: CompiledState<'a>,
    // storage: SelectableStateView<ChainStateDB, InMemoryStateCache<RemoteViewer>>,
    default_syntax: SyntaxChoice,
    //tempdir: TempDir,
    //debug: bool,
    moveos: MoveOS,
}

#[derive(Parser, Debug)]
pub struct MoveOSPublishArgs {}

#[derive(Parser, Debug)]
pub struct MoveOSRunArgs {}

#[derive(Parser, Debug)]
pub struct MoveOSExtraInitArgs {}

#[derive(Parser, Debug)]
pub enum MoveOSSubcommands {
    /// View a object in the Move storage
    #[clap(name = "view_object")]
    ViewObject {
        /// The address of the object
        #[clap(short, long)]
        object_id: ObjectID,
    },
}

impl<'a> MoveOSTestAdapter<'a> for MoveOSTestRunner<'a> {
    type ExtraPublishArgs = MoveOSPublishArgs;
    type ExtraRunArgs = MoveOSRunArgs;
    type Subcommand = MoveOSSubcommands;
    type ExtraInitArgs = MoveOSExtraInitArgs;
    type ExtraValueArgs = ();

    fn compiled_state(&mut self) -> &mut CompiledState<'a> {
        &mut self.compiled_state
    }

    fn default_syntax(&self) -> SyntaxChoice {
        self.default_syntax
    }

    fn init(
        default_syntax: SyntaxChoice,
        pre_compiled_deps: Option<&'a FullyCompiledProgram>,
        task_opt: Option<TaskInput<(InitCommand, Self::ExtraInitArgs)>>,
    ) -> (Self, Option<String>) {
        let additional_mapping = match task_opt.map(|t| t.command) {
            Some((InitCommand { named_addresses }, _)) => {
                verify_and_create_named_address_mapping(named_addresses).unwrap()
            }
            None => BTreeMap::new(),
        };

        let moveos_store = MoveOSStore::mock_moveos_store().unwrap();

        let genesis: &RoochGenesis = &rooch_genesis::ROOCH_GENESIS;
        let mut moveos = MoveOS::new(
            moveos_store,
            genesis.all_natives(),
            genesis.config_for_test.clone(),
        )
        .unwrap();

        moveos.init_genesis(genesis.genesis_txs()).unwrap();

        let mut named_address_mapping = rooch_framework::rooch_framework_named_addresses();
        for (name, addr) in additional_mapping {
            if named_address_mapping.contains_key(&name) {
                panic!(
                    "Invalid init. The named address '{}' is reserved by the rooch-framework",
                    name
                )
            }
            named_address_mapping.insert(name, addr);
            //TODO find a better way to create account storage
            moveos
                .state()
                .create_account_storage(addr.into_inner())
                .unwrap();
        }

        // Apply new modules and add precompiled address mapping
        let mut change_set = ChangeSet::new();
        let table_change_set = StateChangeSet::default();
        if let Some(pre_compiled_lib) = pre_compiled_deps {
            for c in &pre_compiled_lib.compiled {
                if let CompiledUnitEnum::Module(m) = c {
                    // update named_address_mapping
                    if let Some(named_address) = &m.address_name {
                        let name = named_address.value.to_string();
                        let already_assigned_with_different_value = named_address_mapping
                            .get(&name)
                            .filter(|existed| {
                                existed.into_inner() != m.named_module.address.into_inner()
                            })
                            .is_some();
                        if already_assigned_with_different_value {
                            panic!(
                                "Invalid init. The named address '{}' is already assigned with {}",
                                name,
                                named_address_mapping.get(&name).unwrap(),
                            )
                        }
                        named_address_mapping.insert(name, m.named_module.address);
                    }
                    let (_, module_id) = m.module_id();
                    let mut bytes = vec![];
                    m.named_module.module.serialize(&mut bytes).unwrap();
                    let op = Op::New(bytes);
                    change_set.add_module_op(module_id, op).unwrap();
                }
            }
        }
        moveos
            .state()
            .apply_change_set(change_set, table_change_set)
            .unwrap();

        let mut adapter = Self {
            compiled_state: CompiledState::new(named_address_mapping, pre_compiled_deps, None),
            default_syntax,
            moveos,
        };

        //Auto generate interface to Framework modules
        let stdlib_modules = genesis.modules().unwrap();

        for module in stdlib_modules
            .iter()
            .filter(|module| !adapter.compiled_state.is_precompiled_dep(&module.self_id()))
            .collect::<Vec<_>>()
        {
            adapter
                .compiled_state
                .add_and_generate_interface_file(module.clone());
        }
        (adapter, None)
    }

    fn publish_module(
        &mut self,
        module: move_binary_format::CompiledModule,
        _named_addr_opt: Option<move_core_types::identifier::Identifier>,
        _gas_budget: Option<u64>,
        _extra: Self::ExtraPublishArgs,
    ) -> anyhow::Result<(Option<String>, move_binary_format::CompiledModule)> {
        let mut module_bytes = vec![];
        module.serialize(&mut module_bytes)?;

        let id = module.self_id();
        let sender = *id.address();

        let tx = MoveOSTransaction::new_for_test(
            sender,
            MoveAction::new_module_bundle(vec![module_bytes]),
        );
        let verified_tx = self.moveos.verify(tx)?;
        let (_state_root, output) = self.moveos.execute_and_apply(verified_tx)?;
        Ok((Some(tx_output_to_str(output)), module))
    }

    fn execute_script(
        &mut self,
        script: move_binary_format::file_format::CompiledScript,
        type_args: Vec<move_core_types::language_storage::TypeTag>,
        signers: Vec<ParsedAddress>,
        args: Vec<
            <<Self as MoveOSTestAdapter<'a>>::ExtraValueArgs as ParsableValue>::ConcreteValue,
        >,
        _gas_budget: Option<u64>,
        _extra: Self::ExtraRunArgs,
    ) -> anyhow::Result<(
        Option<String>,
        move_vm_runtime::session::SerializedReturnValues,
    )> {
        let mut script_bytes = vec![];
        script.serialize(&mut script_bytes)?;

        let mut signers: Vec<_> = signers
            .into_iter()
            .map(|addr| self.compiled_state().resolve_address(&addr))
            .collect();

        let args = args
            .iter()
            .map(|arg| arg.simple_serialize().unwrap())
            .collect::<Vec<_>>();

        let tx = MoveOSTransaction::new_for_test(
            signers.pop().unwrap(),
            MoveAction::new_script_call(script_bytes, type_args, args),
        );
        let verified_tx = self.moveos.verify(tx)?;
        let (_state_root, output) = self.moveos.execute_and_apply(verified_tx)?;
        //TODO return values
        let value = SerializedReturnValues {
            mutable_reference_outputs: vec![],
            return_values: vec![],
        };
        Ok((Some(tx_output_to_str(output)), value))
    }

    fn call_function(
        &mut self,
        module: &move_core_types::language_storage::ModuleId,
        function: &move_core_types::identifier::IdentStr,
        type_args: Vec<move_core_types::language_storage::TypeTag>,
        signers: Vec<ParsedAddress>,
        args: Vec<
            <<Self as MoveOSTestAdapter<'a>>::ExtraValueArgs as ParsableValue>::ConcreteValue,
        >,
        _gas_budget: Option<u64>,
        _extra: Self::ExtraRunArgs,
    ) -> anyhow::Result<(
        Option<String>,
        move_vm_runtime::session::SerializedReturnValues,
    )> {
        let mut signers: Vec<_> = signers
            .into_iter()
            .map(|addr| self.compiled_state().resolve_address(&addr))
            .collect();

        let args = args
            .iter()
            .map(|arg| arg.simple_serialize().unwrap())
            .collect::<Vec<_>>();
        let function_id = FunctionId::new(module.clone(), function.to_owned());
        let tx = MoveOSTransaction::new_for_test(
            signers.pop().unwrap(),
            MoveAction::new_function_call(function_id, type_args, args),
        );
        let verified_tx = self.moveos.verify(tx)?;
        let (_state_root, output) = self.moveos.execute_and_apply(verified_tx)?;
        debug_assert!(output.status == move_core_types::vm_status::KeptVMStatus::Executed);
        //TODO return values
        let value = SerializedReturnValues {
            mutable_reference_outputs: vec![],
            return_values: vec![],
        };

        Ok((Some(tx_output_to_str(output)), value))
    }

    fn view_data(
        &mut self,
        address: move_core_types::account_address::AccountAddress,
        module: &move_core_types::language_storage::ModuleId,
        resource: &move_core_types::identifier::IdentStr,
        type_args: Vec<move_core_types::language_storage::TypeTag>,
    ) -> anyhow::Result<String> {
        view_resource_in_move_storage(
            self.moveos.moveos_resolver(),
            address,
            module,
            resource,
            type_args,
        )
    }

    fn handle_subcommand(
        &mut self,
        subcommand: TaskInput<Self::Subcommand>,
    ) -> anyhow::Result<Option<String>> {
        match subcommand.command {
            MoveOSSubcommands::ViewObject { object_id } => {
                let resoler = self.moveos.moveos_resolver();
                let object = resoler
                    .get_annotated_object(object_id)?
                    .ok_or_else(|| anyhow::anyhow!("Object with id {} not found", object_id))?;
                //TODO should we bring the AnnotatedObjectView from jsonrpc to test adapter for better json output formatting?
                Ok(Some(format!("{:?}", object)))
            }
        }
    }
}

pub fn run_test(path: &Path) -> Result<(), Box<dyn std::error::Error + 'static>> {
    run_test_impl(path, None)
}

pub fn run_test_impl<'a>(
    path: &Path,
    fully_compiled_program_opt: Option<&'a FullyCompiledProgram>,
) -> Result<(), Box<dyn std::error::Error + 'static>> {
    moveos::moveos_test_runner::run_test_impl::<MoveOSTestRunner>(
        path,
        fully_compiled_program_opt,
        None,
    )
}

pub fn iterate_directory(path: &Path) -> impl Iterator<Item = PathBuf> {
    walkdir::WalkDir::new(path)
        .into_iter()
        .map(::std::result::Result::unwrap)
        .filter(|entry| {
            entry.file_type().is_file()
                && entry
                    .file_name()
                    .to_str()
                    .map_or(false, |s| !s.starts_with('.')) // Skip hidden files
        })
        .map(|entry| entry.path().to_path_buf())
}

// Perform extended checks to ensure that the dependencies of the Move files are compiled correctly.
pub fn run_integration_test_with_extended_check(
    path: &Path,
    fully_compiled_program_opt: Option<&FullyCompiledProgram>,
    data: &BTreeMap<String, String>,
) -> Result<(), Box<dyn std::error::Error + 'static>> {
    let dep_package_path = path.parent().unwrap().parent().unwrap();

    let mut named_account_address_map = BTreeMap::new();
    let _ = data
        .iter()
        .map(|(key, value)| {
            let account_address = NumericalAddress::parse_str(value.as_str()).unwrap();
            named_account_address_map.insert(key.clone(), account_address.into_inner());
        })
        .collect::<Vec<_>>();

    let global_env = build_model(dep_package_path, named_account_address_map, false, None).unwrap();

    let _ = run_extended_checks(&global_env);

    let extended_checks_error = {
        if global_env.diag_count(Severity::Warning) > 0 {
            let mut buffer = Buffer::no_color();
            global_env.report_diag(&mut buffer, Severity::Warning);
            let buffer_output = String::from_utf8_lossy(buffer.as_slice()).to_string();
            let re = Regex::new("(/.*)(.move:[0-9]+:[0-9]+)").unwrap();
            Some(
                re.replace(buffer_output.as_str(), "/tmp/tempfile$2".to_owned())
                    .to_string(),
            )
        } else {
            None
        }
    };

    moveos::moveos_test_runner::run_test_impl::<MoveOSTestRunner>(
        path,
        fully_compiled_program_opt,
        extended_checks_error,
    )
}

fn tx_output_to_str(output: TransactionOutput) -> String {
    //TODO introduce output view, and print json output
    output.status.to_string()
}
