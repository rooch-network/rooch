// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

module rooch_framework::auth_validator_registry {

    use moveos_std::type_info;
    use moveos_std::table::{Self, Table};
    use moveos_std::type_table::{Self, TypeTable};
    use moveos_std::context::{Self, Context};
    use rooch_framework::auth_validator::{Self, AuthValidator};

    friend rooch_framework::genesis;
    friend rooch_framework::builtin_validators;

    const ErrorValidatorUnregistered: u64 = 1;
    const ErrorValidatorAlreadyRegistered: u64 = 2;

    struct AuthValidatorWithType<phantom ValidatorType: store> has key {
        id: u64,
    }

    struct ValidatorRegistry has key {
        /// Number of registered validators
        validator_num: u64,
        validators: Table<u64, AuthValidator>,
        validators_with_type: TypeTable,
    }

    /// Init function called by genesis.
    public(friend) fun genesis_init(ctx: &mut Context, sender: &signer){
        let registry = ValidatorRegistry {
            validator_num: 0,
            validators: context::new_table(ctx),
            validators_with_type: context::new_type_table(ctx),
        };
        context::move_resource_to(ctx, sender, registry);
    }

    #[private_generics(ValidatorType)]
    public fun register<ValidatorType: store>(ctx: &mut Context) : u64{
        register_internal<ValidatorType>(ctx)
    }

    public(friend) fun register_internal<ValidatorType: store>(ctx: &mut Context) : u64{
        let type_info = type_info::type_of<ValidatorType>();
        let module_address = type_info::account_address(&type_info);
        //TODO consider change type_info::module_name to ascii::String.
        let module_name = std::ascii::string(type_info::module_name(&type_info));

        let registry = context::borrow_mut_resource<ValidatorRegistry>(ctx, @rooch_framework);
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

    public fun borrow_validator(ctx: &Context, id: u64): &AuthValidator {
        let registry = context::borrow_resource<ValidatorRegistry>(ctx, @rooch_framework);
        assert!(table::contains(&registry.validators, id), ErrorValidatorUnregistered);
        table::borrow(&registry.validators, id)
    }

    public fun borrow_validator_by_type<ValidatorType: store>(ctx: &Context): &AuthValidator {
        let registry = context::borrow_resource<ValidatorRegistry>(ctx, @rooch_framework);
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
        let ctx = context::new_test_context(@rooch_framework);
        genesis_init(&mut ctx, &sender);
        register<TestAuthValidator>(&mut ctx);
        let validator = borrow_validator_by_type<TestAuthValidator>(&ctx);
        let validator_id = auth_validator::validator_id(validator);
        let validator2 = borrow_validator(&ctx, validator_id);
        let validator2_id = auth_validator::validator_id(validator2);
        assert!(validator_id == validator2_id, 1000);
        context::drop_test_context(ctx);
    }
}
