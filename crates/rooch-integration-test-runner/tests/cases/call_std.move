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
    use std::bcs;
    use moveos_std::bcd;

    fun main(s: signer) {
        let addr = signer::address_of(&s);
        let bytes = bcs::to_bytes(&addr);
        let addr2 = bcd::to_address(bytes);
        assert!(addr == addr2, 0);
    }
}