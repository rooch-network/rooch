//# init --addresses creator=0x42

//# publish
module creator::test1 {
    use moveos_std::context::Context;
    use moveos_std::simple_map::SimpleMap;
    use moveos_std::simple_map;

    #[data_struct]
    struct DisallowedStruct<Key: store, Value: store> has copy, drop {
        simple_map: SimpleMap<Key, Value>,
    }

    #[data_struct(T)]
    public fun f1<T: copy+drop>(_data: T) {
    }

    public fun f2(_ctx: &mut Context) {
        let disallowed_struct = DisallowedStruct {
            simple_map: simple_map::create<u8, u8>()
        };
        f1<DisallowedStruct<u8, u8>>(disallowed_struct);
    }
}

