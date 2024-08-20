// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

module rooch_framework::auth_validator_registry {

    use moveos_std::account;
    use moveos_std::type_info;
    use moveos_std::table::{Self, Table};
    use moveos_std::type_table::{Self, TypeTable};
    use moveos_std::features;
    use moveos_std::core_addresses;
    use rooch_framework::auth_validator::{Self, AuthValidator};

    friend rooch_framework::genesis;
    friend rooch_framework::builtin_validators;

    const ErrorValidatorUnregistered: u64 = 1;
    const ErrorValidatorAlreadyRegistered: u64 = 2;

    struct AuthValidatorWithType<phantom ValidatorType: store> has key,store {
        id: u64,
    }

    struct ValidatorRegistry has key {
        /// Number of registered validators
        validator_num: u64,
        validators: Table<u64, AuthValidator>,
        validators_with_type: TypeTable,
    }

    /// Init function called by genesis.
    public(friend) fun genesis_init(sender: &signer){
        let registry = ValidatorRegistry {
            validator_num: 0,
            validators: table::new(),
            validators_with_type: type_table::new(),
        };
        account::move_resource_to(sender, registry);
    }

    #[private_generics(ValidatorType)]
    /// Register a new validator. This feature not enabled in the mainnet.
    public fun register<ValidatorType: store>() : u64{
        features::ensure_testnet_enabled();
        register_internal<ValidatorType>()
    }

    /// Register a new validator by system. This function is only called by system.
    public fun register_by_system<ValidatorType: store>(system: &signer) : u64{
        core_addresses::assert_system_reserved(system);
        register_internal<ValidatorType>()
    }

    public(friend) fun register_internal<ValidatorType: store>() : u64{
        let type_info = type_info::type_of<ValidatorType>();
        let module_address = type_info::account_address(&type_info);
        let module_name = type_info::module_name(&type_info);

        let registry = account::borrow_mut_resource<ValidatorRegistry>(@rooch_framework);
        let id = registry.validator_num;

        assert!(!type_table::contains<AuthValidatorWithType<ValidatorType>>(&registry.validators_with_type), ErrorValidatorAlreadyRegistered);
        
        let validator_with_type = AuthValidatorWithType<ValidatorType>{
            id,
        };
        type_table::add(&mut registry.validators_with_type, validator_with_type);

        let validator = auth_validator::new_auth_validator(
            id,
            module_address,
            module_name,
        );
        table::add(&mut registry.validators, id, validator);
        
        registry.validator_num = registry.validator_num + 1;
        id
    }

    public fun is_registered<ValidatorType: store>(): bool{
        let registry = account::borrow_resource<ValidatorRegistry>(@rooch_framework);
        type_table::contains<AuthValidatorWithType<ValidatorType>>(&registry.validators_with_type)
    }

    public fun borrow_validator(id: u64): &AuthValidator {
        features::ensure_testnet_enabled();

        let registry = account::borrow_resource<ValidatorRegistry>(@rooch_framework);
        assert!(table::contains(&registry.validators, id), ErrorValidatorUnregistered);
        table::borrow(&registry.validators, id)
    }

    public fun borrow_validator_by_type<ValidatorType: store>(): &AuthValidator {
        features::ensure_testnet_enabled();
        
        let registry = account::borrow_resource<ValidatorRegistry>(@rooch_framework);
        assert!(type_table::contains<AuthValidatorWithType<ValidatorType>>(&registry.validators_with_type), ErrorValidatorUnregistered);
        let validator_with_type = type_table::borrow<AuthValidatorWithType<ValidatorType>>(&registry.validators_with_type);
        assert!(table::contains(&registry.validators, validator_with_type.id), ErrorValidatorUnregistered);
        table::borrow(&registry.validators, validator_with_type.id)
    }


    #[test_only]
    struct TestAuthValidator has store{
    }
    #[test(sender=@rooch_framework)]
    fun test_registry(sender: signer){
        features::init_and_enable_all_features_for_test();
        genesis_init(&sender);
        register<TestAuthValidator>();
        let validator = borrow_validator_by_type<TestAuthValidator>();
        let validator_id = auth_validator::validator_id(validator);
        let validator2 = borrow_validator(validator_id);
        let validator2_id = auth_validator::validator_id(validator2);
        assert!(validator_id == validator2_id, 1000);
    }
}
