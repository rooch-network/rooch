module rooch_examples::Data {
    struct Box<T: store> has key, store, drop {
        value: T
    }

    struct Data has key, store, drop, copy {
        v: u64,
    }

    #[private_generics(T3, T2)]
    public fun create_box<T1: store, T2, T3>(value: T1): Box<T1> {
        Box<T1> { value: value }
    }

    public fun box_value<T: copy+store>(box: &Box<T>): T {
        *&box.value
    }

    public fun run() {
        let data = Data{ v: 123 };
        let box_val = create_box<Data, Data, Box<u32>>(data);
        let _ = box_value(&box_val);
    }
}
