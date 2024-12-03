use move_core_types::account_address::AccountAddress;
use move_core_types::ident_str;
use move_core_types::identifier::IdentStr;
use move_core_types::language_storage::ModuleId;
use move_core_types::value::MoveValue;
use moveos_types::move_std::string::MoveString;
use moveos_types::move_types::FunctionId;
use moveos_types::moveos_std::object::ObjectID;
use moveos_types::state::MoveState;
use moveos_types::transaction::FunctionCall;

pub const INVITATION_MODULE_NAME: &IdentStr = ident_str!("invitation");

pub const INVITER_CLAIM_FAUCET: &IdentStr = ident_str!("claim_from_faucet");

pub const CLAIM_FROM_TWITTER_FUNCTION: &IdentStr = ident_str!("claim_from_twitter");
pub fn claim_from_faucet_function_call(
    module_address: AccountAddress,
    faucet_object_id: ObjectID,
    invitation_object_id: ObjectID,
    claimer: AccountAddress,
    utxo_ids: Vec<ObjectID>,
    inviter: AccountAddress,
) -> FunctionCall {
    FunctionCall {
        function_id: FunctionId::new(
            ModuleId::new(module_address, INVITATION_MODULE_NAME.to_owned()),
            INVITER_CLAIM_FAUCET.to_owned(),
        ),
        ty_args: vec![],
        args: vec![
            faucet_object_id.to_move_value().simple_serialize().unwrap(),
            invitation_object_id
                .to_move_value()
                .simple_serialize()
                .unwrap(),
            MoveValue::Address(claimer).simple_serialize().unwrap(),
            MoveValue::Vector(utxo_ids.iter().map(|id| id.to_move_value()).collect())
                .simple_serialize()
                .unwrap(),
            MoveValue::Address(inviter).simple_serialize().unwrap(),
        ],
    }
}

pub fn claim_from_twitter_function_call(
    module_address: AccountAddress,
    tweet_id: String,
    inviter: AccountAddress,
) -> FunctionCall {
    FunctionCall {
        function_id: FunctionId::new(
            ModuleId::new(module_address, INVITATION_MODULE_NAME.to_owned()),
            CLAIM_FROM_TWITTER_FUNCTION.to_owned(),
        ),
        ty_args: vec![],
        args: vec![
            MoveString::from(tweet_id)
                .to_move_value()
                .simple_serialize()
                .unwrap(),
            MoveValue::Address(inviter).simple_serialize().unwrap(),
        ],
    }
}
