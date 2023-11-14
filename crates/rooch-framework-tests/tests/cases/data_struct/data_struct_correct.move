//# init --addresses creator=0x42

//# publish
module creator::test {
    use moveos_std::context::Context;

    #[data_struct]
    struct Inner has copy,drop {
        f_u8: u8,
    }

    #[data_struct]
    struct AllowedStruct has copy,drop {
       f_u64: u64,
       f_address: address,
       f_bool: bool,
       f_str: std::string::String,
       f_custom: Inner,
    }

    #[data_struct(T)]
    public fun f1<T: copy+drop>(_ctx: &mut Context) {
    }

    public fun f2(ctx: &mut Context) {
        f1<AllowedStruct>(ctx);
    }
}

