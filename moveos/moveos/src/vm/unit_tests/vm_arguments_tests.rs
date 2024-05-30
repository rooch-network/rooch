// Copyright (c) The Diem Core Contributors
// Copyright (c) The Move Contributors
// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use crate::gas::table::{initial_cost_schedule, MoveOSGasMeter};
use crate::vm::moveos_vm::MoveOSVM;
use anyhow::Error;
use move_binary_format::file_format::empty_module;
use move_binary_format::{
    errors::VMResult,
    file_format::{
        AbilitySet, AddressIdentifierIndex, Bytecode, CodeUnit, CompiledModule, CompiledScript,
        FieldDefinition, FunctionDefinition, FunctionHandle, FunctionHandleIndex, IdentifierIndex,
        ModuleHandle, ModuleHandleIndex, Signature, SignatureIndex, SignatureToken,
        StructDefinition, StructFieldInformation, StructHandle, StructHandleIndex, TableIndex,
        TypeSignature, Visibility,
    },
};
use move_core_types::identifier::{IdentStr, Identifier};
use move_core_types::metadata::Metadata;
use move_core_types::vm_status::StatusType;
use move_core_types::{
    account_address::AccountAddress,
    language_storage::{ModuleId, StructTag, TypeTag},
    resolver::{ModuleResolver, ResourceResolver},
    u256::U256,
    value::{serialize_values, MoveValue},
    vm_status::StatusCode,
};
use move_vm_runtime::config::VMConfig;
use moveos_types::moveos_std::object::{ObjectEntity, RootObjectEntity};
use moveos_types::state::KeyState;
use moveos_types::state_resolver::{StateKV, StatelessResolver};
use moveos_types::transaction::FunctionCall;
use moveos_types::{
    move_types::FunctionId, moveos_std::object::ObjectID, moveos_std::tx_context::TxContext,
    state::State, state_resolver::StateResolver, transaction::MoveAction,
};
use std::collections::HashMap;

// make a script with a given signature for main.
fn make_script(parameters: Signature) -> Vec<u8> {
    let mut blob = vec![];
    let mut signatures = vec![Signature(vec![])];
    let parameters_idx = match signatures
        .iter()
        .enumerate()
        .find(|(_, s)| *s == &parameters)
    {
        Some((idx, _)) => SignatureIndex(idx as TableIndex),
        None => {
            signatures.push(parameters);
            SignatureIndex((signatures.len() - 1) as TableIndex)
        }
    };
    CompiledScript {
        version: move_binary_format::file_format_common::VERSION_MAX,
        module_handles: vec![],
        struct_handles: vec![],
        function_handles: vec![],

        function_instantiations: vec![],

        signatures,

        identifiers: vec![],
        address_identifiers: vec![],
        constant_pool: vec![],
        metadata: vec![],

        type_parameters: vec![],
        parameters: parameters_idx,
        code: CodeUnit {
            locals: SignatureIndex(0),
            code: vec![Bytecode::LdU64(0), Bytecode::Abort],
        },
    }
    .serialize(&mut blob)
    .expect("script must serialize");
    blob
}

// make a script with an external function that has the same signature as
// the main. That allows us to pass resources and make the verifier happy that
// they are consumed.
// Dependencies check happens after main signature check, so we should expect
// a signature check error.
fn make_script_with_non_linking_structs(parameters: Signature) -> Vec<u8> {
    let mut blob = vec![];
    let mut signatures = vec![Signature(vec![])];
    let parameters_idx = match signatures
        .iter()
        .enumerate()
        .find(|(_, s)| *s == &parameters)
    {
        Some((idx, _)) => SignatureIndex(idx as TableIndex),
        None => {
            signatures.push(parameters);
            SignatureIndex((signatures.len() - 1) as TableIndex)
        }
    };
    CompiledScript {
        version: move_binary_format::file_format_common::VERSION_MAX,
        module_handles: vec![ModuleHandle {
            address: AddressIdentifierIndex(0),
            name: IdentifierIndex(0),
        }],
        struct_handles: vec![StructHandle {
            module: ModuleHandleIndex(0),
            name: IdentifierIndex(1),
            abilities: AbilitySet::EMPTY,
            type_parameters: vec![],
        }],
        function_handles: vec![FunctionHandle {
            module: ModuleHandleIndex(0),
            name: IdentifierIndex(2),
            parameters: SignatureIndex(1),
            return_: SignatureIndex(0),
            type_parameters: vec![],
        }],

        function_instantiations: vec![],

        signatures,

        identifiers: vec![
            Identifier::new("one").unwrap(),
            Identifier::new("two").unwrap(),
            Identifier::new("three").unwrap(),
        ],
        address_identifiers: vec![AccountAddress::random()],
        constant_pool: vec![],
        metadata: vec![],

        type_parameters: vec![],
        parameters: parameters_idx,
        code: CodeUnit {
            locals: SignatureIndex(0),
            code: vec![Bytecode::LdU64(0), Bytecode::Abort],
        },
    }
    .serialize(&mut blob)
    .expect("script must serialize");
    blob
}

pub(crate) fn make_module_with_function(
    visibility: Visibility,
    is_entry: bool,
    parameters: Signature,
    return_: Signature,
    type_parameters: Vec<AbilitySet>,
) -> (CompiledModule, Identifier) {
    let function_name = Identifier::new("foo").unwrap();
    let mut signatures = vec![Signature(vec![])];
    let parameters_idx = match signatures
        .iter()
        .enumerate()
        .find(|(_, s)| *s == &parameters)
    {
        Some((idx, _)) => SignatureIndex(idx as TableIndex),
        None => {
            signatures.push(parameters);
            SignatureIndex((signatures.len() - 1) as TableIndex)
        }
    };
    let return_idx = match signatures.iter().enumerate().find(|(_, s)| *s == &return_) {
        Some((idx, _)) => SignatureIndex(idx as TableIndex),
        None => {
            signatures.push(return_);
            SignatureIndex((signatures.len() - 1) as TableIndex)
        }
    };
    let module = CompiledModule {
        version: move_binary_format::file_format_common::VERSION_MAX,
        self_module_handle_idx: ModuleHandleIndex(0),
        module_handles: vec![ModuleHandle {
            address: AddressIdentifierIndex(0),
            name: IdentifierIndex(0),
        }],
        struct_handles: vec![StructHandle {
            module: ModuleHandleIndex(0),
            name: IdentifierIndex(1),
            abilities: AbilitySet::EMPTY,
            type_parameters: vec![],
        }],
        function_handles: vec![FunctionHandle {
            module: ModuleHandleIndex(0),
            name: IdentifierIndex(2),
            parameters: parameters_idx,
            return_: return_idx,
            type_parameters,
        }],
        field_handles: vec![],
        friend_decls: vec![],

        struct_def_instantiations: vec![],
        function_instantiations: vec![],
        field_instantiations: vec![],

        signatures,

        identifiers: vec![
            Identifier::new("M").unwrap(),
            Identifier::new("X").unwrap(),
            function_name.clone(),
        ],
        address_identifiers: vec![AccountAddress::random()],
        constant_pool: vec![],
        metadata: vec![],

        struct_defs: vec![StructDefinition {
            struct_handle: StructHandleIndex(0),
            field_information: StructFieldInformation::Declared(vec![FieldDefinition {
                name: IdentifierIndex(1),
                signature: TypeSignature(SignatureToken::Bool),
            }]),
        }],
        function_defs: vec![FunctionDefinition {
            function: FunctionHandleIndex(0),
            visibility,
            is_entry,
            acquires_global_resources: vec![],
            code: Some(CodeUnit {
                locals: SignatureIndex(0),
                code: vec![Bytecode::LdU64(0), Bytecode::Abort],
            }),
        }],
    };
    (module, function_name)
}

// make a script function with a given signature for main.
pub(crate) fn make_script_function(signature: Signature) -> (CompiledModule, Identifier) {
    make_module_with_function(
        Visibility::Public,
        true,
        signature,
        Signature(vec![]),
        vec![],
    )
}

pub(crate) struct RemoteStore {
    root: RootObjectEntity,
    modules: HashMap<ModuleId, Vec<u8>>,
}

impl RemoteStore {
    pub(crate) fn new() -> Self {
        Self {
            root: ObjectEntity::genesis_root_object(),
            modules: HashMap::new(),
        }
    }

    fn add_module(&mut self, compiled_module: CompiledModule) {
        let id = compiled_module.self_id();
        let mut bytes = vec![];
        compiled_module.serialize(&mut bytes).unwrap();
        self.modules.insert(id, bytes);
    }
}

impl ModuleResolver for RemoteStore {
    fn get_module_metadata(&self, _module_id: &ModuleId) -> Vec<Metadata> {
        todo!()
    }

    fn get_module(&self, module_id: &ModuleId) -> Result<Option<Vec<u8>>, Error> {
        Ok(self.modules.get(module_id).cloned())
    }
}

impl ResourceResolver for RemoteStore {
    fn get_resource_with_metadata(
        &self,
        _address: &AccountAddress,
        _typ: &StructTag,
        _metadata: &[Metadata],
    ) -> Result<(Option<Vec<u8>>, usize), Error> {
        todo!()
    }
}

impl StatelessResolver for RemoteStore {
    fn get_field_at(
        &self,
        _state_root: moveos_types::h256::H256,
        _key: &KeyState,
    ) -> anyhow::Result<Option<State>, anyhow::Error> {
        Ok(None)
    }

    fn list_fields_at(
        &self,
        _state_root: moveos_types::h256::H256,
        _cursor: Option<KeyState>,
        _limit: usize,
    ) -> anyhow::Result<Vec<StateKV>> {
        Ok(vec![])
    }
}

impl StateResolver for RemoteStore {
    fn get_field(
        &self,
        _handle: &ObjectID,
        _key: &KeyState,
    ) -> anyhow::Result<Option<State>, anyhow::Error> {
        Ok(None)
    }

    fn list_fields(
        &self,
        _handle: &ObjectID,
        _cursor: Option<KeyState>,
        _limit: usize,
    ) -> anyhow::Result<Vec<StateKV>, anyhow::Error> {
        todo!()
    }
    fn root_object(&self) -> &RootObjectEntity {
        &self.root
    }
}

fn combine_signers_and_args(
    signers: Vec<AccountAddress>,
    non_signer_args: Vec<Vec<u8>>,
) -> Vec<Vec<u8>> {
    signers
        .into_iter()
        .map(|s| MoveValue::Signer(s).simple_serialize().unwrap())
        .chain(non_signer_args)
        .collect()
}

fn call_script_with_args_ty_args_signers(
    script: Vec<u8>,
    non_signer_args: Vec<Vec<u8>>,
    ty_args: Vec<TypeTag>,
    signers: Vec<AccountAddress>,
) -> VMResult<()> {
    let moveos_vm = MoveOSVM::new(vec![], VMConfig::default()).unwrap();
    let remote_view = RemoteStore::new();
    let ctx = TxContext::random_for_testing_only();
    let cost_table = initial_cost_schedule(None);
    let mut gas_meter = MoveOSGasMeter::new(cost_table, ctx.max_gas_amount);
    gas_meter.set_metering(false);
    let mut session = moveos_vm.new_session(&remote_view, ctx, gas_meter);

    let script_action = MoveAction::new_script_call(
        script,
        ty_args,
        combine_signers_and_args(signers, non_signer_args),
    );
    let verified_action = session.verify_move_action(script_action)?;
    session.execute_move_action(verified_action)
}

fn call_script(script: Vec<u8>, args: Vec<Vec<u8>>) -> VMResult<()> {
    call_script_with_args_ty_args_signers(script, args, vec![], vec![])
}

fn call_script_function_with_args_ty_args_signers(
    module: CompiledModule,
    function_name: Identifier,
    non_signer_args: Vec<Vec<u8>>,
    ty_args: Vec<TypeTag>,
    signers: Vec<AccountAddress>,
) -> VMResult<()> {
    let moveos_vm = MoveOSVM::new(vec![], VMConfig::default()).unwrap();
    let mut remote_view = RemoteStore::new();
    let id = module.self_id();
    remote_view.add_module(module);
    let ctx = TxContext::random_for_testing_only();
    let cost_table = initial_cost_schedule(None);
    let mut gas_meter = MoveOSGasMeter::new(cost_table, ctx.max_gas_amount);
    gas_meter.set_metering(false);
    let mut session: crate::vm::moveos_vm::MoveOSSession<'_, '_, RemoteStore, MoveOSGasMeter> =
        moveos_vm.new_session(&remote_view, ctx, gas_meter);

    let function_action = MoveAction::new_function_call(
        FunctionId::new(id, function_name),
        ty_args,
        combine_signers_and_args(signers, non_signer_args),
    );
    let verified_action = session.verify_move_action(function_action)?;
    session.execute_move_action(verified_action)
}

fn call_script_function(
    module: CompiledModule,
    function_name: Identifier,
    args: Vec<Vec<u8>>,
) -> VMResult<()> {
    call_script_function_with_args_ty_args_signers(module, function_name, args, vec![], vec![])
}

// these signatures used to be bad, but there are no bad signatures for scripts at the VM
fn deprecated_bad_signatures() -> Vec<Signature> {
    vec![
        // struct in signature
        Signature(vec![SignatureToken::Struct(StructHandleIndex(0))]),
        // struct in signature
        Signature(vec![SignatureToken::Struct(StructHandleIndex(0))]),
        // reference to struct in signature
        Signature(vec![SignatureToken::MutableReference(Box::new(
            SignatureToken::Struct(StructHandleIndex(0)),
        ))]),
        // vector of struct in signature
        Signature(vec![SignatureToken::Vector(Box::new(
            SignatureToken::Struct(StructHandleIndex(0)),
        ))]),
        // vector of vector of struct in signature
        Signature(vec![SignatureToken::Vector(Box::new(
            SignatureToken::Vector(Box::new(SignatureToken::Struct(StructHandleIndex(0)))),
        ))]),
        // reference to vector in signature
        Signature(vec![SignatureToken::Reference(Box::new(
            SignatureToken::Vector(Box::new(SignatureToken::Struct(StructHandleIndex(0)))),
        ))]),
        // reference to vector in signature
        Signature(vec![SignatureToken::Reference(Box::new(
            SignatureToken::U64,
        ))]),
    ]
}

fn good_signatures_and_arguments() -> Vec<(Signature, Vec<MoveValue>)> {
    vec![
        // U128 arg
        (
            Signature(vec![SignatureToken::U128]),
            vec![MoveValue::U128(0)],
        ),
        // U8 arg
        (Signature(vec![SignatureToken::U8]), vec![MoveValue::U8(0)]),
        // U16 arg
        (
            Signature(vec![SignatureToken::U16]),
            vec![MoveValue::U16(0)],
        ),
        // U32 arg
        (
            Signature(vec![SignatureToken::U32]),
            vec![MoveValue::U32(0)],
        ),
        // U256 arg
        (
            Signature(vec![SignatureToken::U256]),
            vec![MoveValue::U256(U256::zero())],
        ),
        // All constants
        (
            Signature(vec![SignatureToken::Vector(Box::new(SignatureToken::Bool))]),
            vec![MoveValue::Vector(vec![
                MoveValue::Bool(false),
                MoveValue::Bool(true),
            ])],
        ),
        // All constants
        (
            Signature(vec![
                SignatureToken::Bool,
                SignatureToken::Vector(Box::new(SignatureToken::U8)),
                SignatureToken::Address,
            ]),
            vec![
                MoveValue::Bool(true),
                MoveValue::vector_u8(vec![0, 1]),
                MoveValue::Address(AccountAddress::random()),
            ],
        ),
        // vector<vector<address>>
        (
            Signature(vec![
                SignatureToken::Bool,
                SignatureToken::Vector(Box::new(SignatureToken::U8)),
                SignatureToken::Vector(Box::new(SignatureToken::Vector(Box::new(
                    SignatureToken::Address,
                )))),
            ]),
            vec![
                MoveValue::Bool(true),
                MoveValue::vector_u8(vec![0, 1]),
                MoveValue::Vector(vec![
                    MoveValue::Vector(vec![
                        MoveValue::Address(AccountAddress::random()),
                        MoveValue::Address(AccountAddress::random()),
                    ]),
                    MoveValue::Vector(vec![
                        MoveValue::Address(AccountAddress::random()),
                        MoveValue::Address(AccountAddress::random()),
                    ]),
                    MoveValue::Vector(vec![
                        MoveValue::Address(AccountAddress::random()),
                        MoveValue::Address(AccountAddress::random()),
                    ]),
                ]),
            ],
        ),
        //
        // Vector arguments
        //
        // empty vector
        (
            Signature(vec![SignatureToken::Vector(Box::new(
                SignatureToken::Address,
            ))]),
            vec![MoveValue::Vector(vec![])],
        ),
        // one elem vector
        (
            Signature(vec![SignatureToken::Vector(Box::new(
                SignatureToken::Address,
            ))]),
            vec![MoveValue::Vector(vec![MoveValue::Address(
                AccountAddress::random(),
            )])],
        ),
        // multiple elems vector
        (
            Signature(vec![SignatureToken::Vector(Box::new(
                SignatureToken::Address,
            ))]),
            vec![MoveValue::Vector(vec![
                MoveValue::Address(AccountAddress::random()),
                MoveValue::Address(AccountAddress::random()),
                MoveValue::Address(AccountAddress::random()),
                MoveValue::Address(AccountAddress::random()),
                MoveValue::Address(AccountAddress::random()),
            ])],
        ),
        // empty vector of vector
        (
            Signature(vec![SignatureToken::Vector(Box::new(
                SignatureToken::Vector(Box::new(SignatureToken::U8)),
            ))]),
            vec![MoveValue::Vector(vec![])],
        ),
        // multiple element vector of vector
        (
            Signature(vec![SignatureToken::Vector(Box::new(
                SignatureToken::Vector(Box::new(SignatureToken::U8)),
            ))]),
            vec![MoveValue::Vector(vec![
                MoveValue::vector_u8(vec![0, 1]),
                MoveValue::vector_u8(vec![2, 3]),
                MoveValue::vector_u8(vec![4, 5]),
            ])],
        ),
    ]
}

fn mismatched_cases() -> Vec<(Signature, Vec<MoveValue>, StatusCode)> {
    vec![
        // Too few args
        (
            Signature(vec![SignatureToken::U64]),
            vec![],
            StatusCode::NUMBER_OF_ARGUMENTS_MISMATCH,
        ),
        // Too many args
        (
            Signature(vec![SignatureToken::Bool]),
            vec![
                MoveValue::Bool(false),
                MoveValue::Bool(false),
                MoveValue::Bool(false),
            ],
            StatusCode::NUMBER_OF_ARGUMENTS_MISMATCH,
        ),
        // Vec<bool> passed for vec<address>
        (
            Signature(vec![SignatureToken::Vector(Box::new(
                SignatureToken::Address,
            ))]),
            vec![MoveValue::Vector(vec![MoveValue::Bool(true)])],
            StatusCode::FAILED_TO_DESERIALIZE_ARGUMENT,
        ),
        // u128 passed for vec<address>
        (
            Signature(vec![SignatureToken::Vector(Box::new(
                SignatureToken::Address,
            ))]),
            vec![MoveValue::U128(12)],
            StatusCode::FAILED_TO_DESERIALIZE_ARGUMENT,
        ),
        // u8 passed for vector<vector<u8>>
        (
            Signature(vec![SignatureToken::Vector(Box::new(
                SignatureToken::Vector(Box::new(SignatureToken::U8)),
            ))]),
            vec![MoveValue::U8(12)],
            StatusCode::FAILED_TO_DESERIALIZE_ARGUMENT,
        ),
    ]
}

type TestCases = Vec<(
    Signature,
    Vec<MoveValue>,
    Vec<AccountAddress>,
    Option<StatusCode>,
)>;

fn general_cases() -> TestCases {
    // In Moveos, the `Context` will auto resolve singers,
    // so we don't need to pass a signer argument.
    vec![
        // too few signers (0)
        (
            Signature(vec![SignatureToken::Signer, SignatureToken::Signer]),
            vec![],
            vec![],
            None,
        ),
        // too few signers (1)
        (
            Signature(vec![SignatureToken::Signer, SignatureToken::Signer]),
            vec![],
            vec![AccountAddress::random()],
            Some(StatusCode::NUMBER_OF_ARGUMENTS_MISMATCH),
        ),
        // too much signers (3)
        (
            Signature(vec![SignatureToken::Signer, SignatureToken::Signer]),
            vec![],
            vec![
                AccountAddress::random(),
                AccountAddress::random(),
                AccountAddress::random(),
            ],
            Some(StatusCode::NUMBER_OF_ARGUMENTS_MISMATCH),
        ),
        // correct number of signers (2)
        (
            Signature(vec![SignatureToken::Signer, SignatureToken::Signer]),
            vec![],
            vec![AccountAddress::random(), AccountAddress::random()],
            Some(StatusCode::NUMBER_OF_ARGUMENTS_MISMATCH),
        ),
        // too many signers (1) in a script that expects 0 is no longer ok
        (
            Signature(vec![SignatureToken::U8]),
            vec![MoveValue::U8(0)],
            vec![AccountAddress::random()],
            Some(StatusCode::NUMBER_OF_ARGUMENTS_MISMATCH),
        ),
        // signer
        (
            Signature(vec![
                SignatureToken::Signer,
                SignatureToken::Bool,
                SignatureToken::Address,
            ]),
            vec![
                MoveValue::Bool(false),
                MoveValue::Address(AccountAddress::random()),
            ],
            vec![],
            None,
        ),
    ]
}

#[test]
fn check_script() {
    let _ = tracing_subscriber::fmt::try_init();
    //
    // Bad signatures
    //
    for signature in deprecated_bad_signatures() {
        let num_args = signature.0.len();
        let dummy_args = vec![MoveValue::Bool(false); num_args];
        let script = make_script_with_non_linking_structs(signature);
        let status = call_script(script, serialize_values(&dummy_args))
            .err()
            .unwrap()
            .major_status();
        assert_eq!(status, StatusCode::LINKER_ERROR);
    }

    //
    // Good signatures
    //
    for (signature, args) in good_signatures_and_arguments() {
        // Body of the script is just an abort, so `ABORTED` means the script was accepted and ran
        let expected_status = StatusCode::ABORTED;
        let script = make_script(signature);
        assert_eq!(
            call_script(script, serialize_values(&args))
                .err()
                .unwrap()
                .major_status(),
            expected_status
        )
    }

    //
    // Mismatched Cases
    //
    for (signature, args, error) in mismatched_cases() {
        let script = make_script(signature.clone());
        let result = call_script(script, serialize_values(&args));
        assert!(
            result.is_err(),
            "signature: {:?}, args: {:?}",
            signature,
            args
        );
        let result_error = result.err().unwrap();
        assert_eq!(
            result_error.major_status(),
            error,
            "signature: {:?}, args: {:?}, expected_status: {:?}, result_error: {:?}",
            signature,
            args,
            error,
            result_error
        );
    }

    for (signature, args, signers, expected_status_opt) in general_cases() {
        // Body of the script is just an abort, so `ABORTED` means the script was accepted and ran
        let expected_status = expected_status_opt.unwrap_or(StatusCode::ABORTED);
        let script = make_script(signature.clone());
        let result =
            call_script_with_args_ty_args_signers(script, serialize_values(&args), vec![], signers);
        assert!(
            result.is_err(),
            "signature: {:?}, args: {:?}",
            signature,
            args
        );
        let result_error = result.err().unwrap();
        assert_eq!(
            result_error.major_status(),
            expected_status,
            "signature: {:?}, args: {:?}, expected_status: {:?}, result_error: {:?}",
            signature,
            args,
            expected_status,
            result_error
        );
    }
}

#[test]
#[allow(dead_code)]
fn check_script_function() {
    let _ = tracing_subscriber::fmt::try_init();
    //
    // Bad signatures
    //
    for signature in deprecated_bad_signatures() {
        let num_args = signature.0.len();
        let dummy_args = vec![MoveValue::Bool(false); num_args];
        let (module, function_name) = make_script_function(signature);
        let res = call_script_function(module, function_name, serialize_values(&dummy_args))
            .err()
            .unwrap();
        assert_eq!(res.major_status(), StatusCode::ABORTED,)
    }

    //
    // Good signatures
    //
    for (signature, args) in good_signatures_and_arguments() {
        // Body of the script is just an abort, so `ABORTED` means the script was accepted and ran
        let expected_status = StatusCode::ABORTED;
        let (module, function_name) = make_script_function(signature);
        assert_eq!(
            call_script_function(module, function_name, serialize_values(&args))
                .err()
                .unwrap()
                .major_status(),
            expected_status
        )
    }

    //
    // Mismatched Cases
    //
    for (signature, args, error) in mismatched_cases() {
        let (module, function_name) = make_script_function(signature);
        assert_eq!(
            call_script_function(module, function_name, serialize_values(&args))
                .err()
                .unwrap()
                .major_status(),
            error
        );
    }

    for (signature, args, signers, expected_status_opt) in general_cases() {
        // Body of the script is just an abort, so `ABORTED` means the script was accepted and ran
        let expected_status = expected_status_opt.unwrap_or(StatusCode::ABORTED);
        let (module, function_name) = make_script_function(signature);
        assert_eq!(
            call_script_function_with_args_ty_args_signers(
                module,
                function_name,
                serialize_values(&args),
                vec![],
                signers
            )
            .err()
            .unwrap()
            .major_status(),
            expected_status
        );
    }

    //
    // Non script visible
    // DEPRECATED this check must now be done by the adapter
    //
    // public
    let (module, function_name) = make_module_with_function(
        Visibility::Public,
        true,
        Signature(vec![]),
        Signature(vec![]),
        vec![],
    );
    assert_eq!(
        call_script_function_with_args_ty_args_signers(
            module,
            function_name,
            vec![],
            vec![],
            vec![],
        )
        .err()
        .unwrap()
        .major_status(),
        StatusCode::ABORTED,
    );
    // private
    let (module, function_name) = make_module_with_function(
        Visibility::Private,
        true,
        Signature(vec![]),
        Signature(vec![]),
        vec![],
    );
    assert_eq!(
        call_script_function_with_args_ty_args_signers(
            module,
            function_name,
            vec![],
            vec![],
            vec![],
        )
        .err()
        .unwrap()
        .major_status(),
        StatusCode::ABORTED,
    );
}

//TODO directly use MoveOS to test this
#[test]
fn call_missing_item() {
    let _ = tracing_subscriber::fmt::try_init();
    let module = empty_module();
    let id = &module.self_id();
    let function_name = IdentStr::new("foo").unwrap();
    // missing module
    let moveos_vm = MoveOSVM::new(vec![], VMConfig::default()).unwrap();
    let mut remote_view = RemoteStore::new();
    let ctx = TxContext::random_for_testing_only();
    let cost_table = initial_cost_schedule(None);
    let mut gas_meter = MoveOSGasMeter::new(cost_table, ctx.max_gas_amount);
    gas_meter.set_metering(false);
    let mut session = moveos_vm.new_session(&remote_view, ctx.clone(), gas_meter);
    let func_call = FunctionCall::new(
        FunctionId::new(id.clone(), function_name.into()),
        vec![],
        Vec::<Vec<u8>>::new(),
    );
    let error = session
        .execute_function_bypass_visibility(func_call.clone())
        .err()
        .unwrap();
    assert_eq!(error.major_status(), StatusCode::LINKER_ERROR);
    assert_eq!(error.status_type(), StatusType::Verification);
    drop(session);

    // missing function
    remote_view.add_module(module);
    let cost_table = initial_cost_schedule(None);
    let mut gas_meter = MoveOSGasMeter::new(cost_table, ctx.max_gas_amount);
    gas_meter.set_metering(false);
    let mut session = moveos_vm.new_session(&remote_view, ctx, gas_meter);
    let error = session
        .execute_function_bypass_visibility(func_call)
        .err()
        .unwrap();
    assert_eq!(
        error.major_status(),
        StatusCode::FUNCTION_RESOLUTION_FAILURE
    );
    assert_eq!(error.status_type(), StatusType::Verification);
}
