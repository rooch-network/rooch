module rooch_examples::module1 {
    // struct Data {
    //     v: u64
    // }

    struct Box<T> has drop {
        v: T
    }

    #[private_generics(T1, T2)]
    public fun new_box<T1, T2, T3>(value: T1): Box<T1> {
        Box { v: value }
    }

    public fun get_box_value<T: copy>(box: &Box<T>): T {
        box.v
    }

    #[test]
    fun test1() {
        // let data = Data { v: 123};
        let box = new_box<u32, u64, u128>(123);
        assert!(get_box_value(&box) == 123, 1000);
    }
}