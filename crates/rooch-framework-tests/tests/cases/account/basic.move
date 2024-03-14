//# init --addresses genesis=0x1

//TODO currently, we auto create account for init addresses, so I remove the bob=0x42 from the init addresses.
//In the future, if we create account by faucet, we can keep the init named address. 

//TODO create account by faucet

//create account by bob self
//# run --signers genesis
script {
    use rooch_framework::account;
    
    fun main(_sender: signer) {
        account::create_account_entry(@0x42);
    }
}

//check
//# run --signers 0x42
script {
    use moveos_std::account;
    
    fun main(_sender: signer) {
        assert!(account::exists_at(@0x42), 0);
    }
}
