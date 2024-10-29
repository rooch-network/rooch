//# init --addresses creator=0x42

//# publish
module creator::test {
    // INVALID_PARAM_SINGER_COUNT
    entry public fun test_entry_multiple_signer(_acount1: &signer, _acount2: &signer, _acount3: &signer){
        
    }
}
