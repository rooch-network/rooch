/// This module contains the error code for auth_validator module
/// The auth_validator implementation should contain the following functions
/// public fun validate(ctx: &StorageContext, payload: vector<u8>)
/// fun pre_execute(ctx: &mut StorageContext)
/// fun post_execute(ctx: &mut StorageContext)
module rooch_framework::auth_validator{
    use std::error;
    
    /// The AuthKey in transaction's authenticator do not match with the sender's account auth key
    const EValidateInvalidAccountAuthKey: u64 = 1001;
    /// InvalidAuthenticator, include invalid signature
    const EValidateInvalidAuthenticator: u64 = 1002;


    public fun error_invalid_account_auth_key(): u64 {
       error::invalid_argument(EValidateInvalidAccountAuthKey) 
    }

    public fun error_invalid_authenticator(): u64 {
       error::invalid_argument(EValidateInvalidAuthenticator) 
    }

}