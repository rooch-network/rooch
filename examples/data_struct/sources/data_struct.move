module rooch_examples::data_struct {
    use moveos_std::context::Context;

    #[data_struct]
    struct Inner has copy,drop{
        f_u8: u8,
    }

    #[data_struct]
    struct Outer{
        f_u64: u64,
        f_address: address,
        f_bool: bool,
        f_str: std::string::String,
        f_custom: Inner,
    }

    #[data_struct(T)]
    fun f1<T: drop>(_ctx: &Context, _inner: T): bool{
        false
    }

    fun f2(_ctx: &Context) {
        let inner = Inner {f_u8: 1};
        f1(_ctx, inner);
    }
}
