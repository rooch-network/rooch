// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use crate::cli_types::{CommandAction, WalletContextOptions};
use async_trait::async_trait;
use clap::Parser;
use codespan_reporting::diagnostic::Severity;
use move_cli::{base::test, Move};
use move_command_line_common::address::NumericalAddress;
use move_command_line_common::parser::NumberFormat;
use move_unit_test::extensions::{set_after_execution_hook, set_extension_hook};
use move_vm_runtime::native_extensions::NativeContextExtensions;
use moveos::vm::data_cache;
use moveos_config::DataDirPath;
use moveos_object_runtime::runtime::{ObjectRuntime, ObjectRuntimeContext};
use moveos_stdlib::natives::moveos_stdlib::{
    event::NativeEventContext, move_module::NativeModuleContext,
};
use moveos_store::{load_feature_store_object, MoveOSStore};
use moveos_types::moveos_std::event::{Event, EventID, TransactionEvent};
use moveos_types::moveos_std::object::ObjectMeta;
use moveos_types::{moveos_std::tx_context::TxContext, state_resolver::RootObjectResolver};
use moveos_verifier::build::build_model_with_test_attr;
use moveos_verifier::metadata::run_extended_checks;
use once_cell::sync::Lazy;
use parking_lot::RwLock;
use rooch_genesis::FrameworksGasParameters;
use rooch_rpc_api::jsonrpc_types::event_view::EventView;
use rooch_rpc_api::jsonrpc_types::{ObjectChangeView, OpView, StateChangeSetView};
use rooch_types::error::{RoochError, RoochResult};
use rooch_types::genesis_config;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::rc::Rc;
use std::{collections::BTreeMap, path::PathBuf};
use termcolor::Buffer;
use tokio::runtime::Runtime;

#[derive(Parser)]
#[group(skip)]
pub struct TestCommand {
    #[clap(flatten)]
    pub test: test::Test,

    /// Named addresses for the move binary
    ///
    /// Example: alice=0x1234, bob=0x5678
    ///
    /// Note: This will fail if there are duplicates in the Move.toml file remove those first.
    #[clap(long, value_parser = crate::utils::parse_map::< String, String >, default_value = "")]
    pub(crate) named_addresses: BTreeMap<String, String>,

    #[clap(flatten)]
    config_options: WalletContextOptions,

    #[clap(flatten)]
    move_args: Move,

    /// Return command outputs in json format
    #[clap(long, default_value = "false")]
    json: bool,

    /// Print events emitted during test execution
    #[clap(long, default_value = "true")]
    print_events: bool,

    /// Print objects created or modified during test execution
    #[clap(long, default_value = "false")]
    print_objects: bool,
}

#[async_trait]
impl CommandAction<Option<Value>> for TestCommand {
    async fn execute(self) -> RoochResult<Option<Value>> {
        let context = self.config_options.build()?;

        let mut build_config = self.move_args.build_config;
        build_config
            .additional_named_addresses
            .extend(context.parse_and_resolve_addresses(self.named_addresses)?);

        let root_path = self
            .move_args
            .package_path
            .clone()
            .unwrap_or_else(|| PathBuf::from("."));

        build_config.dev_mode = true;

        let resolution_graph = build_config
            .clone()
            .resolution_graph_for_package(&root_path, &mut Vec::new())?;

        let mut additional_named_address = BTreeMap::new();
        let _: Vec<_> = resolution_graph
            .extract_named_address_mapping()
            .map(|(name, addr)| {
                (additional_named_address.insert(
                    name.to_string(),
                    NumericalAddress::new(addr.into_bytes(), NumberFormat::Hex).into_inner(),
                ),)
            })
            .collect();

        let global_env = build_model_with_test_attr(&root_path, additional_named_address, None)?;

        let _ = run_extended_checks(&global_env);

        if global_env.diag_count(Severity::Warning) > 0 {
            let mut buffer = Buffer::ansi();
            global_env.report_diag(&mut buffer, Severity::Warning);
            let buffer_output = String::from_utf8_lossy(buffer.as_slice()).to_string();
            eprintln!("{}", buffer_output);
            if global_env.has_errors() {
                return Err(RoochError::from(anyhow::Error::msg(
                    "extended checks failed",
                )));
            }
        }

        //TODO define gas metering
        let cost_table = move_vm_test_utils::gas_schedule::INITIAL_COST_SCHEDULE.clone();
        let gas_parameter = FrameworksGasParameters::initial();
        let natives = gas_parameter.all_natives();

        // Setup extension hook for native runtime
        set_extension_hook(Box::new(|extensions| {
            new_moveos_natives_runtime(extensions);
        }));

        // Setup the after-execution hook to process and report test results
        let json_output = self.json;
        let print_events = self.print_events;
        let print_objects = self.print_objects;

        set_after_execution_hook(Box::new(move |mut extensions| {
            if !print_events && !print_objects {
                return;
            }

            // Collect test results
            let mut results = TestResults::new();

            // Collect events if requested
            if print_events {
                let event_context = extensions.remove::<NativeEventContext>();

                results.events = event_context
                    .into_events()
                    .into_iter()
                    .enumerate()
                    .map(
                        |(index, (type_tag, event_handle_id, event_data))| TransactionEvent {
                            event_type: type_tag,
                            event_data,
                            event_index: index as u64,
                            event_handle_id,
                        },
                    )
                    .map(|tx_event| {
                        let event_id =
                            EventID::new(tx_event.event_handle_id.clone(), tx_event.event_index);
                        Event::new_with_event_id(event_id, tx_event)
                    })
                    .map(|event| {
                        //TODO apply the changeset to the moveos store and support view_resource
                        // let event_move_value = MoveValueAnnotator::new(&resolver)
                        //     .view_resource(event.event_type(), event.event_data()).unwrap();
                        // let annotated_event = AnnotatedEvent::new(event, event_move_value);
                        EventView::from(event)
                    })
                    .collect();
            }

            // Collect change set if requested
            if print_objects {
                let obj_runtime_ctx = extensions.remove::<ObjectRuntimeContext>();
                let runtime = obj_runtime_ctx.into_inner();

                // Get the change set from the runtime
                let (_, change_set) = data_cache::into_change_set(runtime).unwrap();
                results.change_set = Some(change_set.into());
            }

            // Output results in the requested format
            if json_output {
                let json = results.to_json();
                println!("{}", serde_json::to_string_pretty(&json).unwrap());
            } else {
                results.write_text(&mut std::io::stdout());
            }
        }));

        self.test.execute(
            self.move_args.package_path,
            build_config,
            natives,
            Some(cost_table),
            // print the result to stderr, stdout for the execution event and objects
            &mut std::io::stderr(),
        )?;
        Ok(None)
    }
}

/// Structure to hold test results that can be formatted in different ways
#[derive(Serialize, Deserialize, Debug)]
struct TestResults {
    events: Vec<EventView>,
    change_set: Option<StateChangeSetView>,
}

impl TestResults {
    fn new() -> Self {
        Self {
            events: Vec::new(),
            change_set: None,
        }
    }

    /// Convert the test results to JSON format
    fn to_json(&self) -> Value {
        serde_json::to_value(self).unwrap_or_default()
    }

    /// Write test results in text format
    fn write_text<W: std::io::Write + ?Sized>(&self, writer: &mut W) {
        // Write events if available
        if !self.events.is_empty() {
            writeln!(writer, "\nEvents emitted during test execution:").unwrap();
            for event in self.events.iter() {
                writeln!(
                    writer,
                    "Event {}: {} - {}",
                    event.event_index,
                    event.event_type,
                    event
                        .decoded_event_data
                        .as_ref()
                        .map(|data| serde_json::to_string(data).unwrap_or_default())
                        .unwrap_or_default(),
                )
                .unwrap();
            }
            writeln!(writer).unwrap();
        }

        // Write change set if available
        if let Some(change_set) = &self.change_set {
            writeln!(writer, "\nObjects created/modified during test execution:").unwrap();

            // Write root object info

            writeln!(writer, "  State root: {}", change_set.state_root).unwrap();
            writeln!(writer, "  Size: {}", change_set.global_size).unwrap();

            // Process all changes
            if !change_set.changes.is_empty() {
                writeln!(writer, "\nObject changes:").unwrap();
                for change in &change_set.changes {
                    Self::write_object_change(writer, change, 0);
                }
            }

            writeln!(writer).unwrap();
        }
    }

    /// Recursively write object changes in text format
    fn write_object_change<W: std::io::Write + ?Sized>(
        writer: &mut W,
        change: &ObjectChangeView,
        depth: usize,
    ) {
        let indent = "  ".repeat(depth);
        let change_type = match &change.value {
            Some(op) => match op {
                OpView::New(_) => "New",
                OpView::Modify(_) => "Modified",
                OpView::Delete => "Deleted",
            },
            None => "Metadata only",
        };

        writeln!(
            writer,
            "{}Object ID: {} ({})",
            indent, change.metadata.id, change_type
        )
        .unwrap();
        writeln!(writer, "{}  Owner: {}", indent, change.metadata.owner).unwrap();
        writeln!(writer, "{}  Size: {}", indent, change.metadata.size).unwrap();
        writeln!(
            writer,
            "{}  Created at: {}",
            indent, change.metadata.created_at
        )
        .unwrap();
        writeln!(
            writer,
            "{}  Updated at: {}",
            indent, change.metadata.updated_at
        )
        .unwrap();

        // Recursively process child fields
        if !change.fields.is_empty() {
            writeln!(writer, "{}  Child objects:", indent).unwrap();
            for child_change in &change.fields {
                Self::write_object_change(writer, child_change, depth + 2);
            }
        }
    }
}

static MOVEOSSTORE: Lazy<(MoveOSStore, DataDirPath)> = Lazy::new(|| {
    let runtime = Runtime::new()
        .expect("Failed to create Tokio runtime when mock moveos store in move unit test");
    runtime.block_on(async { MoveOSStore::mock_moveos_store().unwrap() })
});

static RESOLVER: Lazy<Box<RootObjectResolver<MoveOSStore>>> = Lazy::new(|| {
    Box::new(RootObjectResolver::new(
        ObjectMeta::genesis_root(),
        &MOVEOSSTORE.0,
    ))
});

#[allow(clippy::arc_with_non_send_sync)]
fn new_moveos_natives_runtime(ext: &mut NativeContextExtensions) {
    let resolver = Lazy::force(&RESOLVER).as_ref();
    let object_runtime = Rc::new(RwLock::new(ObjectRuntime::genesis(
        TxContext::random_for_testing_only(),
        ObjectMeta::genesis_root(),
        resolver,
        genesis_config::G_LOCAL_CONFIG.genesis_objects.clone(),
    )));
    let feature_store = load_feature_store_object(resolver);
    let table_ext = ObjectRuntimeContext::new(object_runtime, feature_store);
    let module_ext = NativeModuleContext::new(resolver);
    let event_ext = NativeEventContext::default();
    ext.add(table_ext);
    ext.add(module_ext);
    ext.add(event_ext);
}
