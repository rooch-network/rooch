module rooch_framework::auth_validator_registry{

    use std::error;
    use moveos_std::type_info;
    use moveos_std::table::{Self, Table};
    use moveos_std::type_table::{Self, TypeTable};
    use moveos_std::account_storage;
    use moveos_std::storage_context::{Self, StorageContext};
    use rooch_framework::auth_validator::{Self, AuthValidator};

    friend rooch_framework::genesis;
    friend rooch_framework::builtin_validators;

    const EValidatorUnregistered: u64 = 1;
    const EValidatorAlreadyRegistered: u64 = 2;

    

    struct AuthValidatorWithType<phantom ValidatorType: store> has key {
        id: u64,
    }

    struct ValidatorRegistry has key {
        ///How many validators are registered
        validator_num: u64,
        validators: Table<u64, AuthValidator>,
        validators_with_type: TypeTable,
    }

    /// Init function called by genesis.
    public(friend) fun genesis_init(ctx: &mut StorageContext, sender: &signer){
        let registry = ValidatorRegistry {
            validator_num: 0,
            validators: table::new(storage_context::tx_context_mut(ctx)),
            validators_with_type: type_table::new(storage_context::tx_context_mut(ctx)),
        };
        account_storage::global_move_to(ctx, sender, registry);
    }

    #[private_generics(ValidatorType)]
    public fun register<ValidatorType: store>(ctx: &mut StorageContext) : u64{
        register_internal<ValidatorType>(ctx)
    }

    public(friend) fun register_internal<ValidatorType: store>(ctx: &mut StorageContext) : u64{
        let type_info = type_info::type_of<ValidatorType>();
        let module_address = type_info::account_address(&type_info);
        //TODO consider change type_info::module_name to ascii::String.
        let module_name = std::ascii::string(type_info::module_name(&type_info));

        let registry = account_storage::global_borrow_mut<ValidatorRegistry>(ctx, @rooch_framework);
        let id = registry.validator_num;

        assert!(!type_table::contains<AuthValidatorWithType<ValidatorType>>(&registry.validators_with_type), error::already_exists(EValidatorAlreadyRegistered));
        
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

    public fun borrow_validator(ctx: &StorageContext, id: u64): &AuthValidator {
        let registry = account_storage::global_borrow<ValidatorRegistry>(ctx, @rooch_framework);
        table::borrow(&registry.validators, id)
    }

    public fun borrow_validator_by_type<ValidatorType: store>(ctx: &StorageContext): &AuthValidator {
        let registry = account_storage::global_borrow<ValidatorRegistry>(ctx, @rooch_framework);
        assert!(type_table::contains<AuthValidatorWithType<ValidatorType>>(&registry.validators_with_type), error::not_found(EValidatorUnregistered));
        let validator_with_type = type_table::borrow<AuthValidatorWithType<ValidatorType>>(&registry.validators_with_type);
        assert!(table::contains(&registry.validators, validator_with_type.id), error::not_found(EValidatorUnregistered));
        table::borrow(&registry.validators, validator_with_type.id)
    }


    #[test_only]
    struct TestAuthValidator has store{
    }
    #[test(sender=@rooch_framework)]
    fun test_registry(sender: signer){
        let ctx = storage_context::new_test_context(@rooch_framework);
        genesis_init(&mut ctx, &sender);
        register<TestAuthValidator>(&mut ctx);
        let validator = borrow_validator_by_type<TestAuthValidator>(&ctx);
        let validator_id = auth_validator::validator_id(validator);
        let validator2 = borrow_validator(&ctx, validator_id);
        let validator2_id = auth_validator::validator_id(validator2);
        assert!(validator_id == validator2_id, 1000);
        storage_context::drop_test_context(ctx);
    }
}