use move_binary_format::binary_views::BinaryIndexedView;
use move_binary_format::file_format::{Bytecode, FunctionInstantiation, SignatureToken};
use move_core_types::language_storage::ModuleId;
use move_model::ast::Attribute;
use move_model::model::{FunctionEnv, GlobalEnv, Loc, ModuleEnv, QualifiedId, StructId};
use move_model::ty::PrimitiveType;
use move_model::ty::Type;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

const PRIVATE_GENERICS_ATTRIBUTE: &str = "private_generics";

/// Enumeration of potentially known attributes
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct KnownAttribute {
    kind: u8,
    args: Vec<String>,
}

/// V1 of Aptos specific metadata attached to the metadata section of file_format.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct RuntimeModuleMetadataV1 {
    /// Attributes attached to structs.
    pub struct_attributes: BTreeMap<String, Vec<KnownAttribute>>,

    /// Attributes attached to functions, by definition index.
    pub fun_attributes: BTreeMap<String, Vec<KnownAttribute>>,

    /// The correspondence between private generics and their type parameters.
    pub private_generics_indices: BTreeMap<String, Vec<usize>>,
}

impl RuntimeModuleMetadataV1 {
    pub fn is_empty(&self) -> bool {
        self.fun_attributes.is_empty()
            && self.struct_attributes.is_empty()
            && self.private_generics_indices.is_empty()
    }
}

/// Run the extended context checker on target modules in the environment and returns a map
/// from module to extended runtime metadata. Any errors during context checking are reported to
/// `env`. This is invoked after general build succeeds.
pub fn run_extended_checks(env: &GlobalEnv) -> BTreeMap<ModuleId, RuntimeModuleMetadataV1> {
    let mut checker = ExtendedChecker::new(env);
    checker.run();
    checker.output
}

#[derive(Debug)]
struct ExtendedChecker<'a> {
    env: &'a GlobalEnv,
    /// Computed runtime metadata
    output: BTreeMap<ModuleId, RuntimeModuleMetadataV1>,
}

impl<'a> ExtendedChecker<'a> {
    fn new(env: &'a GlobalEnv) -> Self {
        Self {
            env,
            output: BTreeMap::default(),
        }
    }

    fn run(&mut self) {
        for ref module in self.env.get_modules() {
            if module.is_target() {
                self.check_private_generics_functions(module);
                self.check_entry_functions(module);
                //self.check_init_module(module);
            }
        }
    }
}

// ----------------------------------------------------------------------------------
// Private Generic Functions

impl<'a> ExtendedChecker<'a> {
    fn check_private_generics_functions(&mut self, module: &ModuleEnv) {
        let mut type_name_indices: BTreeMap<String, Vec<usize>> = BTreeMap::new();

        // Check every function and if a function has the private_generics attribute,
        // ensure that the function name and the types defined in the private_generics attribute match,
        // for example: #[private_generics(T1, T2)].
        for ref fun in module.get_functions() {
            if !self.has_attribute(fun, PRIVATE_GENERICS_ATTRIBUTE) {
                continue;
            }

            let mut func_type_params_name_list = vec![];
            let type_params = fun.get_named_type_parameters();
            for t in type_params {
                let type_name = self.env.symbol_pool().string(t.0).as_str().to_string();
                func_type_params_name_list.push(type_name);
            }

            if func_type_params_name_list.is_empty() {
                self.env
                    .error(&fun.get_loc(), "Function do not has type parameter.");
            }

            let attributes = fun.get_attributes();

            for attr in attributes {
                if let Attribute::Apply(_, _, types) = attr {
                    if types.is_empty() {
                        self.env.error(
                            &fun.get_loc(),
                            "A type name is needed for private generics.",
                        );
                    }

                    let mut attribute_type_index = vec![];
                    let mut attribute_type_names = vec![];
                    for (idx, type_name) in func_type_params_name_list.iter().enumerate() {
                        let _ = types
                            .iter()
                            .map(|attr| {
                                if let Attribute::Apply(_, name, _) = attr {
                                    let attribute_type_name =
                                        self.env.symbol_pool().string(*name).as_str().to_string();

                                    if attribute_type_name == type_name.as_str() {
                                        attribute_type_index.push(idx);
                                        attribute_type_names.push(attribute_type_name);
                                    }
                                }
                            })
                            .collect::<Vec<_>>();
                    }

                    let _ = types
                        .iter()
                        .map(|attr| {
                            if let Attribute::Apply(_, name, _) = attr {
                                let attribute_type_name =
                                    self.env.symbol_pool().string(*name).as_str().to_string();
                                if !attribute_type_names.contains(&attribute_type_name) {
                                    let func_name = self
                                        .env
                                        .symbol_pool()
                                        .string(fun.get_name())
                                        .as_str()
                                        .to_string();

                                    self.env.error(
                                        &fun.get_loc(),
                                        format!(
                                            "type name {:?} not defined in function {:?}",
                                            attribute_type_name, func_name
                                        )
                                        .as_str(),
                                    );
                                }
                            }
                        })
                        .collect::<Vec<_>>();

                    type_name_indices.insert(
                        self.env
                            .symbol_pool()
                            .string(fun.get_name())
                            .as_str()
                            .to_string(),
                        attribute_type_index.clone(),
                    );
                }
            }
        }

        let module = module.get_verified_module();
        let view = BinaryIndexedView::Module(module);

        // Inspect the bytecode of every function, and if an instruction is CallGeneric,
        // verify that it calls a function with the private_generics attribute as detected earlier.
        // Then, ensure that the generic parameters of the CallGeneric instruction are valid.
        for func_def in &module.function_defs {
            let code = match func_def.code.clone() {
                None => continue,
                Some(code) => code,
            };

            for instr in code.code {
                if let Bytecode::CallGeneric(finst_idx) = instr {
                    let FunctionInstantiation {
                        handle,
                        type_parameters,
                    } = view.function_instantiation_at(finst_idx);

                    let fhandle = view.function_handle_at(*handle);
                    let func_name = view.identifier_at(fhandle.name).to_string();

                    let type_arguments = &view.signature_at(*type_parameters).0;
                    let private_generics_types = type_name_indices.get(func_name.as_str());

                    if let Some(private_generics_types_indices) = private_generics_types {
                        for generic_type_index in private_generics_types_indices {
                            let type_arg = type_arguments.get(*generic_type_index).unwrap();
                            let (defined_in_current_module, struct_name) =
                                is_defined_in_current_module(&view, type_arg);

                            if !defined_in_current_module {
                                panic!(
                                    "{}",
                                    format!(
                                        "resource struct {:?} not defined in current module",
                                        struct_name
                                    )
                                );
                            }
                        }
                    }
                }
            }
        }

        for (private_generics_func_name, types_list) in type_name_indices {
            let type_params_idicies = self
                .output
                .entry(module.self_id().clone())
                .or_default()
                .private_generics_indices
                .entry(private_generics_func_name)
                .or_default();

            let _ = types_list
                .iter()
                .map(|index| type_params_idicies.push(*index))
                .collect::<Vec<_>>();
        }
    }
}

// ----------------------------------------------------------------------------------
// Entry Function

impl<'a> ExtendedChecker<'a> {
    fn check_entry_functions(&mut self, module: &ModuleEnv) {
        for ref fun in module.get_functions() {
            if !fun.is_entry() {
                continue;
            }

            let arg_tys = &fun.get_parameter_types();
            for ty in arg_tys {
                self.check_transaction_input_type(&fun.get_loc(), ty);
            }

            if fun.get_return_count() > 0 {
                self.env
                    .error(&fun.get_loc(), "entry function cannot return values")
            }
        }
    }

    fn check_transaction_input_type(&self, loc: &Loc, ty: &Type) {
        use Type::*;
        match ty {
            Primitive(_) | TypeParameter(_) => {
                // Any primitive type allowed, any parameter expected to instantiate with primitive
            }
            Vector(ety) => {
                // Vectors are allowed if element type is allowed
                self.check_transaction_input_type(loc, ety)
            }
            Struct(mid, sid, _) if self.is_allowed_input_struct(mid.qualified(*sid)) => {
                // Specific struct types are allowed
            }
            Reference(false, bt)
                if matches!(bt.as_ref(), Primitive(PrimitiveType::Signer))
                    || self.is_allowed_reference_types(bt) =>
            {
                // Immutable Reference to signer and specific types is allowed
            }
            Reference(true, bt) if self.is_allowed_reference_types(bt) => {
                // Mutable references to specific types is allowed
            }
            _ => {
                // Everything else is disallowed.
                self.env.error(
                    loc,
                    &format!(
                        "type `{}` is not supported as a parameter type",
                        ty.display(&self.env.get_type_display_ctx())
                    ),
                );
            }
        }
    }

    fn is_allowed_reference_types(&self, bt: &Type) -> bool {
        match bt {
            Type::Struct(mid, sid, _) => {
                if self.is_allowed_input_struct(mid.qualified(*sid)) {
                    return true;
                }
                false
            }
            _ => false,
        }
    }

    fn is_allowed_input_struct(&self, qid: QualifiedId<StructId>) -> bool {
        let name = self.env.get_struct(qid).get_full_name_with_address();
        matches!(
            name.as_str(),
            "0x1::string::String"
                | "0x1::object_id::ObjectID"
                | "0x1::storage_context::StorageContext"
                | "0x1::tx_context::TxContext"
        )
    }
}

// ----------------------------------------------------------------------------------
// Helpers

impl<'a> ExtendedChecker<'a> {
    fn has_attribute(&self, fun: &FunctionEnv, attr_name: &str) -> bool {
        fun.get_attributes().iter().any(|attr| {
            if let Attribute::Apply(_, name, _) = attr {
                self.env.symbol_pool().string(*name).as_str() == attr_name
            } else {
                false
            }
        })
    }
}

fn is_defined_in_current_module(
    view: &BinaryIndexedView,
    type_arg: &SignatureToken,
) -> (bool, String) {
    match type_arg {
        SignatureToken::Struct(idx) | SignatureToken::StructInstantiation(idx, _) => {
            let shandle = view.struct_handle_at(*idx);
            (
                view.self_handle_idx() == Some(shandle.module),
                view.identifier_at(shandle.name).to_string(),
            )
        }
        SignatureToken::TypeParameter(_)
        | SignatureToken::Bool
        | SignatureToken::U8
        | SignatureToken::U16
        | SignatureToken::U32
        | SignatureToken::U64
        | SignatureToken::U128
        | SignatureToken::U256
        | SignatureToken::Address
        | SignatureToken::Vector(_)
        | SignatureToken::Signer
        | SignatureToken::Reference(_)
        | SignatureToken::MutableReference(_) => (false, "".to_string()),
    }
}
