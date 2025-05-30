// Copyright (c) The Diem Core Contributors
// Copyright (c) The Move Contributors
// SPDX-License-Identifier: Apache-2.0

//use crate::sandbox::utils::on_disk_state_view::OnDiskStateView;
use anyhow::{bail, Result};
use colored::Colorize;
use difference::{Changeset, Difference};
use move_binary_format::{
    access::ModuleAccess,
    compatibility::Compatibility,
    errors::VMError,
    file_format::{AbilitySet, CompiledModule, FunctionDefinitionIndex, SignatureToken},
    IndexKind,
};
use move_bytecode_utils::Modules;
use move_command_line_common::files::{FileHash, MOVE_COMPILED_EXTENSION};
use move_compiler::{
    compiled_unit::{CompiledUnit, NamedCompiledModule},
    diagnostics::{self, report_diagnostics, Diagnostic, Diagnostics, FileName},
};
use move_core_types::{
    account_address::AccountAddress,
    effects::{ChangeSet, Event, Op},
    errmap::ErrorMapping,
    language_storage::{ModuleId, TypeTag},
    transaction_argument::TransactionArgument,
    vm_status::{AbortLocation, StatusCode, VMStatus},
};
use move_ir_types::location::Loc;
use move_package::compilation::compiled_package::CompiledUnitWithSource;
use move_resource_viewer::{AnnotatedMoveStruct, MoveValueAnnotator};
use move_vm_test_utils::gas_schedule::Gas;
use std::{
    collections::{BTreeMap, HashMap},
    fs,
    path::Path,
};

pub mod on_disk_state_view;
pub mod package_context;

use move_bytecode_utils::module_cache::GetModule;
use move_vm_test_utils::gas_schedule::{CostTable, GasStatus};
pub use on_disk_state_view::*;
pub use package_context::*;

pub fn get_gas_status(cost_table: &CostTable, gas_budget: Option<u64>) -> Result<GasStatus> {
    let gas_status = if let Some(gas_budget) = gas_budget {
        // TODO(Gas): This should not be hardcoded.
        let max_gas_budget = u64::MAX.checked_div(1000).unwrap();
        if gas_budget >= max_gas_budget {
            bail!("Gas budget set too high; maximum is {}", max_gas_budget)
        }
        GasStatus::new(cost_table, Gas::new(gas_budget))
    } else {
        // no budget specified. Disable gas metering
        GasStatus::new_unmetered()
    };
    Ok(gas_status)
}

pub(crate) fn module(unit: &CompiledUnit) -> Result<&CompiledModule> {
    match unit {
        CompiledUnit::Module(NamedCompiledModule { module, .. }) => Ok(module),
        _ => bail!("Found script in modules -- this shouldn't happen"),
    }
}

pub(crate) fn explain_publish_changeset(changeset: &ChangeSet) {
    // publish effects should contain no resources
    assert!(changeset.resources().next().is_none());
    // total bytes written across all accounts
    let mut total_bytes_written = 0;
    for (addr, name, blob_op) in changeset.modules() {
        match blob_op {
            Op::New(module_bytes) => {
                let bytes_written = addr.len() + name.len() + module_bytes.len();
                total_bytes_written += bytes_written;
                let module_id = ModuleId::new(addr, name.clone());
                println!(
                    "Publishing a new module {} (wrote {:?} bytes)",
                    module_id, bytes_written
                );
            }
            Op::Modify(module_bytes) => {
                let bytes_written = addr.len() + name.len() + module_bytes.len();
                total_bytes_written += bytes_written;
                let module_id = ModuleId::new(addr, name.clone());
                println!(
                    "Updating an existing module {} (wrote {:?} bytes)",
                    module_id, bytes_written
                );
            }
            Op::Delete => {
                panic!("Deleting a module is not supported")
            }
        }
    }
    println!(
        "Wrote {:?} bytes of module ID's and code",
        total_bytes_written
    )
}

// Print a struct with a specified outer indent
fn print_struct_with_indent(value: &AnnotatedMoveStruct, indent: u64) {
    let indent_str: String = (0..indent).map(|_| " ").collect::<String>();
    let value_str = format!("{}", value);
    let lines = value_str.split('\n');
    for line in lines {
        println!("{}{}", indent_str, line)
    }
}

// Print struct diff with a specified outer indent
fn print_struct_diff_with_indent(
    value1: &AnnotatedMoveStruct,
    value2: &AnnotatedMoveStruct,
    indent: u64,
) {
    let indent_str: String = (0..indent).map(|_| " ").collect::<String>();
    let prev_str = format!("{}", value1);
    let new_str = format!("{}", value2);

    let Changeset { diffs, .. } = Changeset::new(&prev_str, &new_str, "\n");

    for diff in diffs {
        match diff {
            Difference::Same(ref x) => {
                let lines = x.split('\n');
                for line in lines {
                    println!(" {}{}", indent_str, line);
                }
            }
            Difference::Add(ref x) => {
                let lines = x.split('\n');
                for line in lines {
                    println!("{}{}{}", "+".green(), indent_str, line.green());
                }
            }
            Difference::Rem(ref x) => {
                let lines = x.split('\n');
                for line in lines {
                    println!("{}{}{}", "-".red(), indent_str, line.red());
                }
            }
        }
    }
}

pub(crate) fn explain_execution_effects(
    changeset: &ChangeSet,
    events: &[Event],
    state: &OnDiskStateView,
) -> Result<()> {
    // execution effects should contain no modules
    assert!(changeset.modules().next().is_none());
    if !events.is_empty() {
        println!("Emitted {:?} events:", events.len());
        // TODO: better event printing
        for (event_key, event_sequence_number, _event_type, event_data) in events {
            println!(
                "Emitted {:?} as the {}th event to stream {:?}",
                event_data, event_sequence_number, event_key
            )
        }
    }
    if !changeset.accounts().is_empty() {
        println!(
            "Changed resource(s) under {:?} address(es):",
            changeset.accounts().len()
        );
    }
    // total bytes written across all accounts
    let mut total_bytes_written = 0;
    for (addr, account) in changeset.accounts() {
        print!("  ");
        if account.resources().is_empty() {
            continue;
        }
        println!(
            "Changed {:?} resource(s) under address {:?}:",
            account.resources().len(),
            addr
        );
        for (struct_tag, write_op) in account.resources() {
            print!("    ");
            let mut bytes_to_write = struct_tag.access_vector().len();
            match write_op {
                Op::New(blob) => {
                    bytes_to_write += blob.len();
                    println!(
                        "Added type {}: {:?} (wrote {:?} bytes)",
                        struct_tag, blob, bytes_to_write
                    );
                    // Print new resource
                    let resource =
                        MoveValueAnnotator::new(state).view_resource(struct_tag, blob)?;
                    print_struct_with_indent(&resource, 6)
                }
                Op::Modify(blob) => {
                    bytes_to_write += blob.len();
                    println!(
                        "Changed type {}: {:?} (wrote {:?} bytes)",
                        struct_tag, blob, bytes_to_write
                    );
                    // Print resource diff
                    let resource_data = state
                        .get_resource_bytes(*addr, struct_tag.clone())?
                        .unwrap();
                    let resource_old =
                        MoveValueAnnotator::new(state).view_resource(struct_tag, &resource_data)?;
                    let resource_new =
                        MoveValueAnnotator::new(state).view_resource(struct_tag, blob)?;

                    print_struct_diff_with_indent(&resource_old, &resource_new, 8)
                }
                Op::Delete => {
                    println!(
                        "Deleted type {} (wrote {:?} bytes)",
                        struct_tag, bytes_to_write
                    );
                    // Print deleted resource
                    let resource_data = state
                        .get_resource_bytes(*addr, struct_tag.clone())?
                        .unwrap();
                    let resource_old =
                        MoveValueAnnotator::new(state).view_resource(struct_tag, &resource_data)?;
                    print_struct_with_indent(&resource_old, 6);
                }
            };
            total_bytes_written += bytes_to_write;
        }
    }
    if total_bytes_written != 0 {
        println!(
            "Wrote {:?} bytes of resource ID's and data",
            total_bytes_written
        );
    }

    Ok(())
}

/// Commit the resources and events modified by a transaction to disk
pub(crate) fn maybe_commit_effects(
    commit: bool,
    changeset: ChangeSet,
    events: Vec<Event>,
    state: &OnDiskStateView,
) -> Result<()> {
    // similar to explain effects, all module publishing happens via save_modules(), so effects
    // shouldn't contain modules
    if commit {
        for (addr, account) in changeset.into_inner() {
            for (struct_tag, blob_op) in account.into_resources() {
                match blob_op {
                    Op::New(blob) | Op::Modify(blob) => {
                        state.save_resource(addr, struct_tag, &blob)?
                    }
                    Op::Delete => state.delete_resource(addr, struct_tag)?,
                }
            }
        }

        for (event_key, event_sequence_number, event_type, event_data) in events {
            state.save_event(&event_key, event_sequence_number, event_type, event_data)?
        }
    } else if !(changeset.resources().next().is_none() && events.is_empty()) {
        println!("Discarding changes; re-run without --dry-run if you would like to keep them.")
    }

    Ok(())
}

pub(crate) fn explain_type_error(
    script_params: &[SignatureToken],
    signers: &[AccountAddress],
    txn_args: &[TransactionArgument],
) {
    use SignatureToken::*;
    let expected_num_signers = script_params
        .iter()
        .filter(|t| match t {
            Reference(r) => r.is_signer(),
            _ => false,
        })
        .count();
    if expected_num_signers != signers.len() {
        println!(
            "Execution failed with incorrect number of signers: script expected {:?}, but found \
             {:?}",
            expected_num_signers,
            signers.len()
        );
        return;
    }

    // TODO: printing type(s) of missing arguments could be useful
    let expected_num_args = script_params.len() - signers.len();
    if expected_num_args != txn_args.len() {
        println!(
            "Execution failed with incorrect number of arguments: script expected {:?}, but found \
             {:?}",
            expected_num_args,
            txn_args.len()
        );
        return;
    }

    // TODO: print more helpful error message pinpointing the (argument, type)
    // pair that didn't match
    println!("Execution failed with type error when binding type arguments to type parameters")
}

pub(crate) fn explain_publish_error(
    error: VMError,
    state: &OnDiskStateView,
    unit: &CompiledUnitWithSource,
) -> Result<()> {
    use StatusCode::*;
    let mut files = HashMap::new();
    let file_contents = std::fs::read_to_string(&unit.source_path)?;
    let file_hash = FileHash::new(&file_contents);
    files.insert(
        file_hash,
        (
            FileName::from(unit.source_path.to_string_lossy()),
            file_contents,
        ),
    );

    let module = module(&unit.unit)?;
    let module_id = module.self_id();
    let error_clone = error.clone();
    match error.into_vm_status() {
        VMStatus::Error {
            status_code: DUPLICATE_MODULE_NAME,
            ..
        } => {
            println!(
                "Module {} exists already. Re-run without --no-republish to publish anyway.",
                module_id
            );
        }
        VMStatus::Error {
            status_code: BACKWARD_INCOMPATIBLE_MODULE_UPDATE,
            ..
        } => {
            println!("Breaking change detected--publishing aborted. Re-run with --ignore-breaking-changes to publish anyway.");

            let old_module = state.get_module_by_id(&module_id)?.unwrap();

            if Compatibility::new(false, true, false, false)
                .check(&old_module, module)
                .is_err()
            {
                // TODO: we could choose to make this more precise by walking the global state and looking for published
                // structs of this type. but probably a bad idea
                println!("Layout API for structs of module {} has changed. Need to do a data migration of published structs", module_id)
            } else if Compatibility::new(true, false, false, false)
                .check(&old_module, module)
                .is_err()
            {
                // TODO: this will report false positives if we *are* simultaneously redeploying all dependent modules.
                // but this is not easy to check without walking the global state and looking for everything
                println!("Linking API for structs/functions of module {} has changed. Need to redeploy all dependent modules.", module_id)
            }
        }
        VMStatus::Error {
            status_code: CYCLIC_MODULE_DEPENDENCY,
            ..
        } => {
            println!(
                "Publishing module {} introduces cyclic dependencies.",
                module_id
            );
            // find all cycles with an iterative DFS
            let all_modules = state.get_all_modules()?;
            let code_cache = Modules::new(&all_modules);

            let mut stack = vec![];
            let mut state = BTreeMap::new();
            state.insert(module_id.clone(), true);
            for dep in module.immediate_dependencies() {
                stack.push((code_cache.get_module(&dep)?, false));
            }

            while let Some((cur, is_exit)) = stack.pop() {
                let cur_id = cur.self_id();
                if is_exit {
                    state.insert(cur_id, false);
                } else {
                    state.insert(cur_id, true);
                    stack.push((cur, true));
                    for next in cur.immediate_dependencies() {
                        if let Some(is_discovered_but_not_finished) = state.get(&next) {
                            if *is_discovered_but_not_finished {
                                let cycle_path: Vec<_> = stack
                                    .iter()
                                    .filter(|(_, is_exit)| *is_exit)
                                    .map(|(m, _)| m.self_id().to_string())
                                    .collect();
                                println!(
                                    "Cycle detected: {} -> {} -> {}",
                                    module_id,
                                    cycle_path.join(" -> "),
                                    module_id,
                                );
                            }
                        } else {
                            stack.push((code_cache.get_module(&next)?, false));
                        }
                    }
                }
            }
            println!("Re-run with --ignore-breaking-changes to publish anyway.")
        }
        VMStatus::Error {
            status_code: MISSING_DEPENDENCY,
            ..
        } => {
            let err_indices = error_clone.indices();
            let mut diags = Diagnostics::new();
            for (ind_kind, table_ind) in err_indices {
                if let IndexKind::FunctionHandle = ind_kind {
                    let native_function = &(module.function_defs())[*table_ind as usize];
                    let fh = module.function_handle_at(native_function.function);
                    let mh = module.module_handle_at(fh.module);
                    let function_source_map = unit
                        .unit
                        .source_map()
                        .get_function_source_map(FunctionDefinitionIndex(*table_ind));
                    if let Ok(map) = function_source_map {
                        let err_string = format!(
                            "Missing implementation for the native function {}::{}",
                            module.identifier_at(mh.name).as_str(),
                            module.identifier_at(fh.name).as_str()
                        );
                        let diag = Diagnostic::new(
                            diagnostics::codes::Declarations::InvalidFunction,
                            (map.definition_location, err_string),
                            Vec::<(Loc, String)>::new(),
                            Vec::<String>::new(),
                        );
                        diags.add(diag);
                    }
                }
            }
            report_diagnostics(&files, diags)
        }
        VMStatus::Error { status_code, .. } => {
            println!("Publishing failed with unexpected error {:?}", status_code)
        }
        VMStatus::Executed | VMStatus::MoveAbort(..) | VMStatus::ExecutionFailure { .. } => {
            unreachable!()
        }
    }

    Ok(())
}

/// Explain an execution error
pub(crate) fn explain_execution_error(
    error_descriptions: &ErrorMapping,
    error: VMError,
    state: &OnDiskStateView,
    script_type_parameters: &[AbilitySet],
    script_parameters: &[SignatureToken],
    vm_type_args: &[TypeTag],
    signers: &[AccountAddress],
    txn_args: &[TransactionArgument],
) -> Result<()> {
    use StatusCode::*;
    match error.into_vm_status() {
        VMStatus::MoveAbort(AbortLocation::Module(id), abort_code) => {
            // try to use move-explain to explain the abort

            print!(
                "Execution aborted with code {} in module {}.",
                abort_code, id
            );

            if let Some(error_desc) =
                error_descriptions.get_explanation(&id.to_string(), abort_code)
            {
                println!(
                    " Abort code details:\nName: {}\nDescription:{}",
                    error_desc.code_name, error_desc.code_description,
                )
            } else {
                println!()
            }
        }
        VMStatus::MoveAbort(AbortLocation::Script, abort_code) => {
            // TODO: map to source code location
            println!(
                "Execution aborted with code {} in transaction script",
                abort_code
            )
        }
        VMStatus::ExecutionFailure {
            status_code,
            location,
            function,
            code_offset,
            ..
        } => {
            let status_explanation = match status_code {
                RESOURCE_ALREADY_EXISTS => "a RESOURCE_ALREADY_EXISTS error (i.e., \
                                            `move_to<T>(account)` when there is already a \
                                            resource of type `T` under `account`)"
                    .to_string(),
                MISSING_DATA => "a RESOURCE_DOES_NOT_EXIST error (i.e., `move_from<T>(a)`, \
                                 `borrow_global<T>(a)`, or `borrow_global_mut<T>(a)` when there \
                                 is no resource of type `T` at address `a`)"
                    .to_string(),
                ARITHMETIC_ERROR => "an arithmetic error (i.e., integer overflow/underflow, \
                                     div/mod by zero, or invalid shift)"
                    .to_string(),
                VECTOR_OPERATION_ERROR => "an error originated from vector operations (i.e., \
                                           index out of bound, pop an empty vector, or unpack a \
                                           vector with a wrong parity)"
                    .to_string(),
                EXECUTION_STACK_OVERFLOW => "an execution stack overflow".to_string(),
                CALL_STACK_OVERFLOW => "a call stack overflow".to_string(),
                OUT_OF_GAS => "an out of gas error".to_string(),
                _ => format!("a {} error", status_code.status_type()),
            };
            // TODO: map to source code location
            let location_explanation = match location {
                AbortLocation::Module(id) => {
                    format!(
                        "{}::{}",
                        id,
                        state.resolve_function(&id, function)?.unwrap()
                    )
                }
                AbortLocation::Script => "script".to_string(),
            };
            println!(
                "Execution failed because of {} in {} at code offset {}",
                status_explanation, location_explanation, code_offset
            )
        }
        VMStatus::Error {
            status_code: NUMBER_OF_TYPE_ARGUMENTS_MISMATCH,
            ..
        } => println!(
            "Execution failed with incorrect number of type arguments: script expected {:?}, but \
             found {:?}",
            script_type_parameters.len(),
            vm_type_args.len()
        ),
        VMStatus::Error {
            status_code: TYPE_MISMATCH,
            ..
        } => explain_type_error(script_parameters, signers, txn_args),
        VMStatus::Error {
            status_code: LINKER_ERROR,
            ..
        } => {
            // TODO: is this the only reason we can see LINKER_ERROR?
            // Can we also see it if someone manually deletes modules in storage?
            println!(
                "Execution failed due to unresolved type argument(s) (i.e., `--type-args \
                 0x1::M:T` when there is no module named M at 0x1 or no type named T in module \
                 0x1::M)"
            );
        }
        VMStatus::Error { status_code, .. } => {
            println!("Execution failed with unexpected error {:?}", status_code)
        }
        VMStatus::Executed => unreachable!(),
    }
    Ok(())
}

/// Return `true` if `path` is a Move bytecode file based on its extension
pub(crate) fn is_bytecode_file(path: &Path) -> bool {
    path.extension()
        .map_or(false, |ext| ext == MOVE_COMPILED_EXTENSION)
}

/// Return `true` if path contains a valid Move bytecode module
pub(crate) fn contains_module(path: &Path) -> bool {
    is_bytecode_file(path)
        && match fs::read(path) {
            Ok(bytes) => CompiledModule::deserialize(&bytes).is_ok(),
            Err(_) => false,
        }
}
