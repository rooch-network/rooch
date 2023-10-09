//# init --addresses creator=0x42

//# publish
module creator::test {

    fun init( _ : & signer){
        
    }
}

//# publish
module creator::test_mut_ref_storage_context {
    use moveos_std::context;

    fun init( _ : &mut context::Context){
        
    }
}

//# publish
module creator::test_signer {

    fun init( _ : signer){
        
    }
}

//# publish
module creator::test_ref_signer {

    fun init( _ : & signer){
        
    }
}