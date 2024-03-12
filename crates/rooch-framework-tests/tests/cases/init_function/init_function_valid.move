//# init --addresses creator=0x42

//# publish
module creator::test {

    fun init(){
        
    }
}

//# publish
module creator::test_signer {

    fun init(_ : signer){
        
    }
}

//# publish
module creator::test_ref_signer {

    fun init(_ : &signer){
        
    }
}