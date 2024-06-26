// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use crate::cli_types::{CommandAction, TransactionOptions, WalletContextOptions};
use async_trait::async_trait;
use clap::Parser;
use move_cli::Move;
use move_core_types::{identifier::Identifier, language_storage::ModuleId};
use moveos_compiler::dependency_order::sort_by_dependency_order;
use moveos_types::{
    addresses::MOVEOS_STD_ADDRESS, move_types::FunctionId, transaction::MoveAction,
};
use moveos_verifier::build::run_verifier;
use moveos_verifier::verifier;
use rooch_key::key_derive::verify_password;
use rooch_key::keystore::account_keystore::AccountKeystore;
use rooch_rpc_api::jsonrpc_types::ExecuteTransactionResponseView;
use rooch_types::address::RoochAddress;
use rooch_types::error::{RoochError, RoochResult};
use rooch_types::transaction::rooch::RoochTransaction;
use rpassword::prompt_password;
use std::collections::BTreeMap;
use std::io::stderr;

#[derive(Parser)]
pub struct Publish {
    #[clap(flatten)]
    context_options: WalletContextOptions,

    #[clap(flatten)]
    move_args: Move,

    #[clap(flatten)]
    tx_options: TransactionOptions,

    /// Named addresses for the move binary
    ///
    /// Example: alice=0x1234, bob=default, alice2=alice
    ///
    /// Note: This will fail if there are duplicates in the Move.toml file remove those first.
    #[clap(long, value_parser=crate::utils::parse_map::<String, String>, default_value = "")]
    pub(crate) named_addresses: BTreeMap<String, String>,

    /// Whether publish modules by `MoveAction::ModuleBundle`?
    /// If not set, publish moduels through Move entry function
    /// `moveos_std::module_store::publish_modules_entry`.
    /// **Deprecated**! Publish modules by `MoveAction::ModuleBundle` is no longer used anymore.
    /// So you should never add this option.
    /// For now, the option is kept for test only.
    #[clap(long)]
    pub by_move_action: bool,
}

#[async_trait]
impl CommandAction<ExecuteTransactionResponseView> for Publish {
    async fn execute(self) -> RoochResult<ExecuteTransactionResponseView> {
        // Build context and handle errors
        let context = self.context_options.build()?;

        // Clone variables for later use
        let package_path = self
            .move_args
            .package_path
            .unwrap_or_else(|| std::env::current_dir().unwrap());
        let config = self.move_args.build_config.clone();
        let mut config = config.clone();

        // Parse named addresses from context and update config
        config.additional_named_addresses =
            context.parse_and_resolve_addresses(self.named_addresses)?;
        let config_cloned = config.clone();

        // Compile the package and run the verifier
        let mut package = compile_with_filter(&package_path, config_cloned.clone())?;
        run_verifier(package_path, config_cloned, &mut package)?;

        // Get the modules from the package
        let modules = package.root_modules_map();
        let empty_modules = modules.iter_modules_owned().is_empty();
        let pkg_address = if !empty_modules {
            let first_module = &modules.iter_modules_owned()[0];
            first_module.self_id().address().to_owned()
        } else {
            return Err(RoochError::MoveCompilationError(format!(
                "compiling move modules error! Is the project or module empty: {:?}",
                empty_modules,
            )));
        };

        // Initialize bundles vector and sort modules by dependency order
        let mut bundles: Vec<Vec<u8>> = vec![];
        let sorted_modules = sort_by_dependency_order(modules.iter_modules())?;
        let resolver = context.get_client().await?;
        // Serialize and collect module binaries into bundles
        verifier::verify_modules(&sorted_modules, &resolver)?;
        for module in sorted_modules {
            let module_address = module.self_id().address().to_owned();
            if module_address != pkg_address {
                return Err(RoochError::MoveCompilationError(format!(
                    "module's address ({:?}) not same as package module address {:?}",
                    module_address,
                    pkg_address.clone(),
                )));
            };
            let mut binary: Vec<u8> = vec![];
            module.serialize(&mut binary)?;
            bundles.push(binary);
        }

        // Validate sender account if provided
        if pkg_address != context.resolve_address(self.tx_options.sender)? {
            return Err(RoochError::CommandArgumentError(
                "--sender-account required and the sender account must be the same as the package address"
                    .to_string(),
            ));
        }

        // Create a sender RoochAddress
        let sender: RoochAddress = pkg_address.into();
        eprintln!("Publish modules to address: {:?}", sender);

        let max_gas_amount: Option<u64> = self.tx_options.max_gas_amount;

        // Prepare and execute the transaction based on the action type
        let tx_result = if !self.by_move_action {
            let args = bcs::to_bytes(&bundles).unwrap();
            let action = MoveAction::new_function_call(
                FunctionId::new(
                    ModuleId::new(
                        MOVEOS_STD_ADDRESS,
                        Identifier::new("module_store".to_owned()).unwrap(),
                    ),
                    Identifier::new("publish_modules_entry".to_owned()).unwrap(),
                ),
                vec![],
                vec![args],
            );

            // Handle transaction with or without authenticator
            match self.tx_options.authenticator {
                Some(authenticator) => {
                    let tx_data = context
                        .build_tx_data(sender, action, max_gas_amount)
                        .await?;
                    let tx = RoochTransaction::new(tx_data, authenticator.into());
                    context.execute(tx).await?
                }
                None => {
                    if context.keystore.get_if_password_is_empty() {
                        context
                            .sign_and_execute(sender, action, None, max_gas_amount)
                            .await?
                    } else {
                        let password =
                            prompt_password("Enter the password to publish:").unwrap_or_default();
                        let is_verified = verify_password(
                            Some(password.clone()),
                            context.keystore.get_password_hash(),
                        )?;

                        if !is_verified {
                            return Err(RoochError::InvalidPasswordError(
                                "Password is invalid".to_owned(),
                            ));
                        }

                        context
                            .sign_and_execute(sender, action, Some(password), max_gas_amount)
                            .await?
                    }
                }
            }
        } else {
            // Handle MoveAction.ModuleBundle case
            let action = MoveAction::ModuleBundle(bundles);

            if context.keystore.get_if_password_is_empty() {
                context
                    .sign_and_execute(sender, action, None, max_gas_amount)
                    .await?
            } else {
                let password =
                    prompt_password("Enter the password to publish:").unwrap_or_default();
                let is_verified =
                    verify_password(Some(password.clone()), context.keystore.get_password_hash())?;

                if !is_verified {
                    return Err(RoochError::InvalidPasswordError(
                        "Password is invalid".to_owned(),
                    ));
                }

                context
                    .sign_and_execute(sender, action, Some(password), max_gas_amount)
                    .await?
            }
        };
        //Directly return the result, the publish transaction may be failed.
        //Caller need to check the `execution_info.status` field.
        Ok(tx_result)
    }
}

use log::info;
use move_command_line_common::address::NumericalAddress;
use move_compiler::expansion::ast::Address;
use move_compiler::parser::ast as P;
use move_compiler::parser::ast::Exp_::Call;
use move_compiler::parser::ast::ModuleName;
use move_compiler::parser::ast::NameAccessChain_::Three;
use move_compiler::parser::ast::SequenceItem_::Bind;
use move_compiler::parser::ast::{Bind_, ModuleIdent_, Var};
use move_compiler::parser::ast::{
    Definition, Exp_, Function, LeadingNameAccess_, ModuleIdent, SequenceItem_, Use,
};
use move_compiler::parser::ast::{FunctionBody_, UseDecl};
use move_compiler::shared::Name;
use move_compiler::{diagnostics, parser, Compiler};
use move_ir_types::location::{sp, Loc};
use move_package::compilation::build_plan::BuildPlan;
use move_package::compilation::compiled_package::CompiledPackage;
use move_package::BuildConfig;
use move_symbol_pool::symbol::Symbol;
use rooch_types::function_arg;
use std::path::PathBuf;
use P::ModuleMember as PM;

fn filter_module_member(module_member: P::ModuleMember) -> Option<P::ModuleMember> {
    use move_compiler::parser::ast::Sequence;
    use move_compiler::parser::ast::Value_ as ASTValue;
    use move_compiler::parser::ast::Var as ASTVar;
    use move_ir_types::location::sp;
    use move_ir_types::sp;

    match module_member {
        PM::Function(func_def) => {
            let Function {
                attributes,
                loc: _loc,
                visibility,
                entry,
                signature,
                acquires,
                inline,
                name,
                body,
            } = func_def.clone();

            let module_addr = sp(
                func_def.loc,
                LeadingNameAccess_::AnonymousAddress(NumericalAddress::parse_str("0x2").unwrap()),
            );

            let module_name: Name = Name::new(func_def.loc, Symbol::from("wasm"));
            let func_name: Name = Name::new(func_def.loc, Symbol::from("native_wasm_test"));

            let start_stmt = {
                let module_address_name = sp(func_def.loc, (module_addr, module_name));
                let mut arg_list = Vec::new();
                arg_list.push(sp(
                    func_def.loc,
                    Exp_::Value(sp(func_def.loc, ASTValue::Num(Symbol::from("123")))),
                ));
                arg_list.push(sp(
                    func_def.loc,
                    Exp_::Value(sp(func_def.loc, ASTValue::Num(Symbol::from("456")))),
                ));
                let args = sp(func_def.loc, arg_list);
                let call_expr = sp(
                    func_def.loc,
                    Call(
                        sp(func_def.loc, Three(module_address_name, func_name)),
                        false,
                        None,
                        args,
                    ),
                );
                let left_var_1: Name = Name::new(func_def.loc, Symbol::from("axxxx111"));
                let left_var_2: Name = Name::new(func_def.loc, Symbol::from("bxxxx111"));
                let mut left_var_list = Vec::new();
                left_var_list.push(sp(func_def.loc, Bind_::Var(Var(left_var_1))));
                left_var_list.push(sp(func_def.loc, Bind_::Var(Var(left_var_2))));
                Bind(sp(func_def.loc, left_var_list), None, Box::from(call_expr))
            };

            let second_stmt = {
                let module_address_name = sp(func_def.loc, (module_addr, module_name));
                let mut arg_list = Vec::new();

                let first_arg_name: Name = Name::new(func_def.loc, Symbol::from("axxxx111"));
                let second_arg_name: Name = Name::new(func_def.loc, Symbol::from("bxxxx111"));

                arg_list.push(sp(func_def.loc, Exp_::Copy(Var(first_arg_name))));
                arg_list.push(sp(func_def.loc, Exp_::Copy(Var(second_arg_name))));
                let args = sp(func_def.loc, arg_list);
                let call_expr = sp(
                    func_def.loc,
                    Call(
                        sp(func_def.loc, Three(module_address_name, func_name)),
                        false,
                        None,
                        args,
                    ),
                );
                sp(func_def.loc, SequenceItem_::Seq(Box::from(call_expr)))
            };

            let func_name = func_def.name.0.to_string();
            //let new_body_sequence = if func_name == "test_counter".to_string() {
                let new_body_sequence = if let FunctionBody_::Defined(sequence) = body.clone().value {
                    let (a, mut sequence_item, c, d) = sequence;

                    info!("11111111 a {:?},", a);
                    info!("33333333 c {:?},", c);
                    info!("44444444 d {:?},", d);
                    sequence_item.insert(0, sp(func_def.loc, start_stmt));
                    sequence_item.push(second_stmt);
                    info!("22222222 b {:?},", sequence_item);
                    let v = (a, sequence_item, c, d);

                    Some(v)
                } else {
                    None
                };
            //} else {
            //    None
            //};
            if new_body_sequence.is_some() {
                let new_body = sp(
                    func_def.loc,
                    FunctionBody_::Defined(new_body_sequence.unwrap()),
                );

                let new_func_def = Function {
                    attributes,
                    loc: _loc,
                    visibility,
                    entry,
                    signature,
                    acquires,
                    inline,
                    name,
                    body: new_body,
                };
                Some(PM::Function(new_func_def))
            } else {
                Some(PM::Function(func_def))
            }
        }
        PM::Struct(struct_def) => Some(PM::Struct(struct_def)),
        PM::Spec(sp!(spec_loc, spec)) => Some(PM::Spec(sp(spec_loc, spec))),
        PM::Use(use_decl) => Some(PM::Use(use_decl)),
        PM::Friend(friend_decl) => Some(PM::Friend(friend_decl)),
        PM::Constant(constant) => Some(PM::Constant(constant)),
    }
}

fn filter_program(program: &mut parser::ast::Program) {
    for source_def in program.source_definitions.iter_mut() {
        let def_cloned = source_def.def.clone();

        let modified_def = {
            match def_cloned {
                Definition::Module(mut module_def) => {
                    let parser::ast::ModuleDefinition {
                        attributes,
                        loc,
                        address,
                        name,
                        is_spec_module,
                        members,
                    } = module_def;

                    let new_members: Vec<_> = members
                        .into_iter()
                        .filter_map(|member| filter_module_member(member))
                        .collect();

                    /*
                    let module_name = Name::new(loc, Symbol::from("wasm"));
                    let mut function_name_path: Vec<(Name, Option<Name>)> = Vec::new();
                    function_name_path
                        .push((Name::new(loc, Symbol::from("native_wasm_test")), None));

                    let addr = sp(
                        loc,
                        LeadingNameAccess_::AnonymousAddress(
                            NumericalAddress::parse_str("0x2").unwrap(),
                        ),
                    );
                    let module_ident = ModuleIdent_ {
                        address: addr,
                        module: ModuleName(module_name),
                    };
                    let module_import_path = ModuleIdent::new(loc, module_ident);

                    let module_import = UseDecl {
                        attributes: vec![],
                        use_: Use::Members(module_import_path, function_name_path),
                    };

                     */

                    //new_members.push(PM::Use(module_import));

                    Definition::Module(parser::ast::ModuleDefinition {
                        attributes,
                        loc,
                        address,
                        name,
                        is_spec_module,
                        members: new_members,
                    })
                }
                Definition::Address(address) => Definition::Address(address.clone()),
                Definition::Script(script) => Definition::Script(script.clone()),
            }
        };

        source_def.def = modified_def;
    }
}

pub fn compile_with_filter(
    path: &PathBuf,
    build_config: BuildConfig,
) -> anyhow::Result<CompiledPackage> {
    let resolved_graph =
        build_config.resolution_graph_for_package(&path, &mut std::io::stdout())?;

    let compiled = BuildPlan::create(resolved_graph)?.compile_with_driver(
        &mut std::io::stdout(),
        Some(6),
        |compiler: Compiler| {
            use move_compiler::PASS_PARSER;
            let (files, pprog_and_comments_res) = compiler.run::<PASS_PARSER>()?;

            let (_comments, stepped) = match pprog_and_comments_res {
                Err(errors) => panic!("compile_with_filter build failed {:?}!", errors),
                Ok(res) => res,
            };

            let (empty_compiler, mut program) = stepped.into_ast();
            filter_program(&mut program);

            let step1_compiler = empty_compiler.at_parser(program);
            let compilation_result = step1_compiler.build();
            let (units, _) = diagnostics::unwrap_or_report_diagnostics(&files, compilation_result);

            Ok((files, units))
        },
    )?;

    Ok(compiled)
}
