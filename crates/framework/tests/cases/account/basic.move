//# init --addresses genesis=0x1 bob=0x42

//TODO create account by faucet

//create account by bob self
//# run --signers genesis
script {
    use mos_framework::account;
    fun main(_sender: signer) {
        account::create_account_entry(@bob);
    }
}

//check
//# run --signers bob
script {
    use mos_framework::account;
    fun main(_sender: signer) {
        assert!(account::exists_at(@bob), 0);
    }
}
