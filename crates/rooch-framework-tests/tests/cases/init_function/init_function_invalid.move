//# init --addresses creator=0x42

//# publish
module creator::test {

    fun init( _: u8 ){
        
    }
}

//# publish
module creator::test {

    fun init( _: u16 ){
        
    }
}

//# publish
module creator::test {

    fun init( _: u32 ){
        
    }
}

//# publish
module creator::test {

    fun init( _: u64 ){
        
    }
}

//# publish
module creator::test {

    fun init( _: u128 ){
        
    }
}

//# publish
module creator::test {

    fun init( _: u256 ){
        
    }
}


//# publish
module creator::test {

    struct Foo has copy, drop {
        x: u64,
    }

    fun init( _foo: Foo ){
        
    }
}

//# publish
module creator::test {
    use moveos_std::object_id;

    fun init( _: object_id::ObjectID ){
        
    }
}

//# publish
module creator::test {
    use std::string;

    fun init( _: string::String ){
        
    }
}

//# publish
module creator::test {
    use std::string;

    fun init( _: string::String ){
        
    }
}

//# publish
module creator::test {
    use std::ascii;

    fun init( _: ascii::String ){
        
    }
}
