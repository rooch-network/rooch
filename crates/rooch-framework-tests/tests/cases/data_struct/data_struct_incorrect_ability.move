//# init --addresses creator=0x42

//# publish
module creator::test {
    

    #[data_struct]
    struct Inner has copy,drop {
        f_u8: u8,
    }

    #[data_struct]
    struct AllowedStruct {
       f_u64: u64,
       f_address: address,
       f_bool: bool,
       f_str: std::string::String,
       f_custom: Inner,
    }

    #[data_struct(T)]
    public fun f1<T>() {
    }

    public fun f2() {
        f1<AllowedStruct>();
    }
}

