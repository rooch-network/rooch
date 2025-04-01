// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use move_command_line_common::address::NumericalAddress;
use move_compiler::command_line::compiler::PASS_COMPILATION;
use move_compiler::command_line::compiler::PASS_INLINING;
use move_compiler::expansion::ast::{self as E};
use move_compiler::shared::known_attributes::KnownAttribute;
use move_compiler::{compiled_unit, FullyCompiledProgram};
use move_model::model::GlobalEnv;
use move_model::options::ModelBuilderOptions;
use move_model::{
    add_move_lang_diagnostics, collect_related_modules_recursive, retrospective_lambda_lifting,
    run_move_checker, run_spec_checker,
};
use std::collections::{BTreeMap, BTreeSet};
use std::rc::Rc;

pub fn build_file_to_module_env(
    pre_compiled_deps: Option<&FullyCompiledProgram>,
    named_address_mapping: BTreeMap<String, NumericalAddress>,
    deps: Vec<String>,
    path: Vec<String>,
    options: ModelBuilderOptions,
    targets_sources: Vec<String>,
    deps_sources: Vec<String>,
) -> anyhow::Result<GlobalEnv> {
    let mut env = GlobalEnv::new();
    env.set_language_version(options.language_version);
    let compile_via_model = options.compile_via_model;
    env.set_extension(options);

    if let Some(fully_compiled_prog) = pre_compiled_deps {
        for package_def in fully_compiled_prog.parser.source_definitions.iter() {
            let fhash = package_def.def.file_hash();
            let (fname, fsrc) = fully_compiled_prog.files.get(&fhash).unwrap();
            let aliases = fully_compiled_prog
                .parser
                .named_address_maps
                .get(package_def.named_address_map)
                .iter()
                .map(|(symbol, addr)| (env.symbol_pool().make(symbol.as_str()), *addr))
                .collect();
            env.add_source(fhash, Rc::new(aliases), fname.as_str(), fsrc, true, true);
        }
    }

    use move_compiler::command_line::compiler::PASS_PARSER;

    // Step 1: parse the program to get comments and a separation of targets and dependencies.
    let flags = move_compiler::Flags::empty().set_sources_shadow_deps(true);
    let (files, comments_and_compiler_res) = move_compiler::Compiler::from_files(
        path,
        deps,
        named_address_mapping,
        flags,
        KnownAttribute::get_all_attribute_names(),
    )
    .set_pre_compiled_lib_opt(pre_compiled_deps)
    .run_with_sources::<PASS_PARSER>(targets_sources.clone(), deps_sources)?;

    let (comment_map, compiler) = match comments_and_compiler_res {
        Err(diags) => {
            // Add source files so that the env knows how to translate locations of parse errors
            let empty_alias = Rc::new(BTreeMap::new());
            for (fhash, (fname, fsrc)) in &files {
                env.add_source(
                    *fhash,
                    empty_alias.clone(),
                    fname.as_str(),
                    fsrc,
                    /* is_target */ true,
                    targets_sources.contains(&fname.to_string()),
                );
            }
            add_move_lang_diagnostics(&mut env, diags);
            return Ok(env);
        }
        Ok(res) => res,
    };
    let (compiler, parsed_prog) = compiler.into_ast();

    // Add source files for targets and dependencies
    let dep_files: BTreeSet<_> = parsed_prog
        .lib_definitions
        .iter()
        .map(|p| p.def.file_hash())
        .collect();

    for member in parsed_prog
        .source_definitions
        .iter()
        .chain(parsed_prog.lib_definitions.iter())
    {
        let fhash = member.def.file_hash();
        let (fname, fsrc) = files.get(&fhash).unwrap();
        let _is_dep = dep_files.contains(&fhash);
        let aliases = parsed_prog
            .named_address_maps
            .get(member.named_address_map)
            .iter()
            .map(|(symbol, addr)| (env.symbol_pool().make(symbol.as_str()), *addr))
            .collect();
        env.add_source(
            fhash,
            Rc::new(aliases),
            fname.as_str(),
            fsrc,
            true,
            targets_sources.contains(&fname.to_string()),
        );
    }

    use itertools::Itertools;

    // If a move file does not contain any definition, it will not appear in `parsed_prog`. Add them explicitly.
    for fhash in files.keys().sorted() {
        if env.get_file_id(*fhash).is_none() {
            let (fname, fsrc) = files.get(fhash).unwrap();
            let _is_dep = dep_files.contains(fhash);
            env.add_source(
                *fhash,
                Rc::new(BTreeMap::new()),
                fname.as_str(),
                fsrc,
                true,
                targets_sources.contains(&fname.to_string()),
            );
        }
    }

    use codespan::ByteIndex;

    // Add any documentation comments found by the Move compiler to the env.
    for (fhash, documentation) in comment_map {
        let file_id = env.get_file_id(fhash).expect("file name defined");
        env.add_documentation(
            file_id,
            documentation
                .into_iter()
                .map(|(idx, s)| (ByteIndex(idx), s))
                .collect(),
        )
    }

    use move_compiler::command_line::compiler::PASS_EXPANSION;
    use move_compiler::parser::ast::{self as P};

    // Step 2: run the compiler up to expansion
    let parsed_prog = {
        let P::Program {
            named_address_maps,
            mut source_definitions,
            lib_definitions,
        } = parsed_prog;
        source_definitions.extend(lib_definitions);
        P::Program {
            named_address_maps,
            source_definitions,
            lib_definitions: vec![],
        }
    };
    let (compiler, expansion_ast) = match compiler.at_parser(parsed_prog).run::<PASS_EXPANSION>() {
        Err(diags) => {
            add_move_lang_diagnostics(&mut env, diags);
            return Ok(env);
        }
        Ok(compiler) => compiler.into_ast(),
    };

    // Extract the module/script closure
    let mut visited_modules = BTreeSet::new();
    for (_, mident, mdef) in &expansion_ast.modules {
        let src_file_hash = mdef.loc.file_hash();
        if !dep_files.contains(&src_file_hash) {
            collect_related_modules_recursive(mident, &expansion_ast.modules, &mut visited_modules);
        }
    }
    for sdef in expansion_ast.scripts.values() {
        let src_file_hash = sdef.loc.file_hash();
        if !dep_files.contains(&src_file_hash) {
            for (_, mident, _neighbor) in &sdef.immediate_neighbors {
                collect_related_modules_recursive(
                    mident,
                    &expansion_ast.modules,
                    &mut visited_modules,
                );
            }
        }
    }
    // Step 3: selective compilation.
    let mut expansion_ast = {
        let E::Program { modules, scripts } = expansion_ast;
        let modules = modules.filter_map(|mident, mut mdef| {
            visited_modules.contains(&mident.value).then(|| {
                mdef.is_source_module = true;
                mdef
            })
        });
        E::Program { modules, scripts }
    };

    if !compile_via_model {
        // Legacy compilation via v1 compiler
        // TODO: eventually remove this code and related helpers

        // Step 4: retrospectively add lambda-lifted function to expansion AST
        let (compiler, inlining_ast) = match compiler
            .at_expansion(expansion_ast.clone())
            .run::<PASS_INLINING>()
        {
            Err(diags) => {
                add_move_lang_diagnostics(&mut env, diags);
                return Ok(env);
            }
            Ok(compiler) => compiler.into_ast(),
        };

        for (loc, module_id, expansion_module) in expansion_ast.modules.iter_mut() {
            match inlining_ast.modules.get_(module_id) {
                None => {
                    env.error(
                        &env.to_loc(&loc),
                        "unable to find matching module in inlining AST",
                    );
                }
                Some(inlining_module) => {
                    retrospective_lambda_lifting(inlining_module, expansion_module);
                }
            }
        }

        // Step 5: Run the compiler from instrumented expansion AST fully to the compiled units

        let units = match compiler
            .at_expansion(expansion_ast.clone())
            .run::<PASS_COMPILATION>()
        {
            Err(diags) => {
                add_move_lang_diagnostics(&mut env, diags);
                return Ok(env);
            }
            Ok(compiler) => {
                let (units, warnings) = compiler.into_compiled_units();
                if !warnings.is_empty() {
                    // NOTE: these diagnostics are just warnings. it should be feasible to continue the
                    // model building here. But before that, register the warnings to the `GlobalEnv`
                    // first so we get a chance to report these warnings as well.
                    add_move_lang_diagnostics(&mut env, warnings);
                }
                units
            }
        };

        // Check for bytecode verifier errors (there should not be any)
        let diags = compiled_unit::verify_units(&units);
        if !diags.is_empty() {
            add_move_lang_diagnostics(&mut env, diags);
            return Ok(env);
        }

        let mut ordered_units = vec![];
        let mut ea = expansion_ast;
        if let Some(pre_compiled) = pre_compiled_deps {
            ordered_units.extend(pre_compiled.clone().compiled);
            let dep_expansion_ast = pre_compiled.clone().expansion.modules;

            for (m_ident, m_def) in dep_expansion_ast {
                ea.modules
                    .add(m_ident, m_def)
                    .expect("expansion modules: duplicate item");
            }
        }
        ordered_units.extend(units);

        // Now that it is known that the program has no errors, run the spec checker on verified units
        // plus expanded AST. This will populate the environment including any errors.
        run_spec_checker(&mut env, ordered_units, ea);
        Ok(env)
    } else {
        // New compilation via model (compiler v2). The expansion AST will be type checked.
        // No bytecode is attached.
        run_move_checker(&mut env, expansion_ast);
        Ok(env)
    }
}
