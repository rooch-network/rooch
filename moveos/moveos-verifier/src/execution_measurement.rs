// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use move_command_line_common::address::NumericalAddress;
use move_compiler::parser::ast as P;
use move_compiler::parser::ast::Exp_::Call;
use move_compiler::parser::ast::NameAccessChain_::Three;
use move_compiler::parser::ast::SequenceItem_::Bind;
use move_compiler::parser::ast::{Bind_, Type_, Var};
use move_compiler::parser::ast::{Definition, Exp_, Function, LeadingNameAccess_};
use move_compiler::parser::ast::{FunctionBody_, NameAccessChain_};
use move_compiler::shared::Name;
use move_compiler::{diagnostics, parser, Compiler};
use move_ir_types::location::sp;
use move_ir_types::sp;
use move_package::compilation::build_plan::BuildPlan;
use move_package::compilation::compiled_package::CompiledPackage;
use move_package::BuildConfig;
use move_symbol_pool::symbol::Symbol;
use std::ops::Deref;
use std::path::Path;
use P::ModuleMember as PM;

fn filter_module_member(
    module_member: P::ModuleMember,
    current_module_address_name: String,
) -> Option<P::ModuleMember> {
    use move_compiler::parser::ast::Value_ as ASTValue;

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

            let full_func_name = format!("{}::{}", current_module_address_name, name);

            let module_addr = sp(
                func_def.loc,
                LeadingNameAccess_::AnonymousAddress(NumericalAddress::parse_str("0x2").unwrap()),
            );

            let module_name: Name = Name::new(func_def.loc, Symbol::from("measurements"));
            let func_name: Name = Name::new(func_def.loc, Symbol::from("inject_parameter"));

            let start_stmt = {
                let module_address_name = sp(func_def.loc, (module_addr, module_name));
                let arg_list = vec![
                    sp(
                        func_def.loc,
                        Exp_::Value(sp(func_def.loc, ASTValue::Num(Symbol::from("0")))),
                    ),
                    sp(
                        func_def.loc,
                        Exp_::Value(sp(func_def.loc, ASTValue::Num(Symbol::from("0")))),
                    ),
                    sp(
                        func_def.loc,
                        Exp_::Value(sp(
                            func_def.loc,
                            ASTValue::ByteString(Symbol::from(full_func_name.clone())),
                        )),
                    ),
                ];
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
                let left_var_1: Name = Name::new(func_def.loc, Symbol::from("ef20de85f8a3__"));
                let left_var_2: Name = Name::new(func_def.loc, Symbol::from("d525d4fafa63__"));
                let left_var_list = vec![
                    sp(func_def.loc, Bind_::Var(Var(left_var_1))),
                    sp(func_def.loc, Bind_::Var(Var(left_var_2))),
                ];
                Bind(sp(func_def.loc, left_var_list), None, Box::from(call_expr))
            };

            let second_stmt = {
                let module_address_name = sp(func_def.loc, (module_addr, module_name));
                let mut arg_list = Vec::new();

                let first_arg_name: Name = Name::new(func_def.loc, Symbol::from("ef20de85f8a3__"));
                let second_arg_name: Name = Name::new(func_def.loc, Symbol::from("d525d4fafa63__"));

                arg_list.push(sp(func_def.loc, Exp_::Move(Var(first_arg_name))));
                arg_list.push(sp(func_def.loc, Exp_::Move(Var(second_arg_name))));
                arg_list.push(sp(
                    func_def.loc,
                    Exp_::Value(sp(
                        func_def.loc,
                        ASTValue::ByteString(Symbol::from(full_func_name.clone())),
                    )),
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
                let left_var_1: Name = Name::new(func_def.loc, Symbol::from("_"));
                let left_var_2: Name = Name::new(func_def.loc, Symbol::from("_"));
                let left_var_list = vec![
                    sp(func_def.loc, Bind_::Var(Var(left_var_1))),
                    sp(func_def.loc, Bind_::Var(Var(left_var_2))),
                ];
                Bind(sp(func_def.loc, left_var_list), None, Box::from(call_expr))
            };

            let new_body_sequence = if let FunctionBody_::Defined(sequence) = body.clone().value {
                let (decl_list, mut sequence_item, loc, return_exp_opt) = sequence;

                let func_return_type = func_def.clone().signature.return_type.value;
                match func_return_type {
                    Type_::Apply(_, _) | Type_::Ref(_, _) => {
                        if let Some(return_exp) = return_exp_opt.deref() {
                            let left_var_list = vec![sp(
                                func_def.loc,
                                Bind_::Var(Var(Name::new(
                                    func_def.loc,
                                    Symbol::from("return__val____"),
                                ))),
                            )];
                            let new_var_bind = Bind(
                                sp(func_def.loc, left_var_list),
                                None,
                                Box::from(return_exp.clone()),
                            );
                            let mut new_seqence_item = sequence_item.clone();
                            new_seqence_item.push(sp(func_def.loc, new_var_bind));

                            let new_return_var =
                                Name::new(func_def.loc, Symbol::from("return__val____"));
                            let new_return_expr = Box::from(Some(sp(
                                func_def.loc,
                                Exp_::Name(
                                    sp(func_def.loc, NameAccessChain_::One(new_return_var)),
                                    None,
                                ),
                            )));

                            new_seqence_item.insert(0, sp(func_def.loc, start_stmt.clone()));
                            new_seqence_item.push(sp(func_def.loc, second_stmt.clone()));

                            Some((decl_list, new_seqence_item, loc, new_return_expr))
                        } else {
                            sequence_item.insert(0, sp(func_def.loc, start_stmt.clone()));
                            sequence_item.push(sp(func_def.loc, second_stmt.clone()));
                            Some((decl_list, sequence_item, loc, return_exp_opt))
                        }
                    }
                    Type_::Fun(_, _) => Some((decl_list, sequence_item, loc, return_exp_opt)),
                    Type_::Unit => {
                        sequence_item.insert(0, sp(func_def.loc, start_stmt.clone()));
                        sequence_item.push(sp(func_def.loc, second_stmt.clone()));
                        Some((decl_list, sequence_item, loc, return_exp_opt))
                    }
                    Type_::Multiple(_) => Some((decl_list, sequence_item, loc, return_exp_opt)),
                }
            } else {
                None
            };

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
                Definition::Module(module_def) => {
                    let parser::ast::ModuleDefinition {
                        attributes,
                        loc,
                        address,
                        name,
                        is_spec_module,
                        members,
                    } = module_def;

                    if let Some(sp!(_loc, LeadingNameAccess_::AnonymousAddress(address))) = address
                    {
                        if address.to_string() == "0x1" {
                            continue;
                        }
                    }

                    if let Some(sp!(_loc, LeadingNameAccess_::Name(name))) = address {
                        if name.value.to_string() == "std" {
                            continue;
                        }
                    }

                    let module_address_name = format!("{}::{}", address.unwrap().clone(), name);

                    let new_members: Vec<_> = members
                        .into_iter()
                        .filter_map(|member| {
                            filter_module_member(member, module_address_name.clone())
                        })
                        .collect();

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
    path: &Path,
    build_config: BuildConfig,
) -> anyhow::Result<CompiledPackage> {
    let resolved_graph = build_config.resolution_graph_for_package(path, &mut std::io::stdout())?;

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
