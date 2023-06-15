module rooch_examples::box {
    struct Box<T: store> has key, store, drop {
        value: T
    }

    struct Data has key, store, drop, copy {
        v: u64,
    }

    #[private_generics(T1)]
    public fun create_box<T1: store>(value: T1): Box<T1> {
        Box<T1> { value: value }
    }

    public fun box_value<T: copy+store>(box: &Box<T>): T {
        *&box.value
    }


    #[test]
    fun test() {
        let data = Data{ v: 123 };
        let box_val = create_box<Data>(data);
        assert!(box_value(&box_val).v == 123, 0);   
    }
}
