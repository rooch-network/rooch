//# init --addresses test=0x42

//# publish
module test::m{
    public fun return_reference(): &u8 {
        &1u8
    }
}


//# publish
module test::m2{
    
    public fun return_reference(): &u8 {
        Self::return_reference_native()
    }

    native fun return_reference_native(): &u8;
}