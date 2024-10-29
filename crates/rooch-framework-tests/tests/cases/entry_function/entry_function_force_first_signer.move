//# init --addresses creator=0x42

//# publish
module creator::test {
    // INVALID_FIRST_ARGUMENT_IS_NOT_SIGNER
    entry public fun test_entry_first_signer(_arg: u64, _acount2: &signer){

    }
}
