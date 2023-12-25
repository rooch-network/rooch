//# init --addresses creator=0x42

//# publish
module creator::test1 {
    use moveos_std::context::Context;

    struct NonDataStruct has drop,store{
    }

    #[data_struct]
    struct DisallowedStruct has drop{
        value: NonDataStruct,
    }

    #[data_struct(T)]
    public fun f1<T: drop>(_data: T) {
    }

    public fun f2(_ctx: &mut Context) {
        let disallowed_struct = DisallowedStruct {
            value: NonDataStruct {},
        };
        f1<DisallowedStruct>(disallowed_struct);
    }
}

