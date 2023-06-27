//# init --addresses creator=0x42

//call move_std
//# run --signers creator
script {
    use std::signer;

    fun main(s: signer) {
        let _addr = signer::address_of(&s);
    }
}


//call moveos_std
//# run --signers creator
script {
    use std::signer;
    use moveos_std::bcs;

    fun main(s: signer) {
        let addr = signer::address_of(&s);
        let bytes = bcs::to_bytes(&addr);
        let addr2 = bcs::to_address(bytes);
        assert!(addr == addr2, 0);
    }
}