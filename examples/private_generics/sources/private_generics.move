module rooch_examples::box {
    struct Box<T: store> has key, store, drop {
        value: T
    }

    struct Data has key, store, drop, copy {
        v: u64,
    }

    #[private_generics(T2)]
    public fun create_box<T1: store, T2>(value: T1): Box<T1> {
        Box<T1> { value: value }
    }

    public fun box_value<T: copy+store>(box: &Box<T>): T {
        *&box.value
    }

    public fun run() {
        let data = Data{ v: 123 };
        let box_val = create_box<Data, Box<u32>>(data);
        let _ = box_value(&box_val);
    }

    #[test]
    fun test() {
        let data = Data{ v: 123 };
        let box_val = create_box<Data, Box<Data>>(data);
        assert!(box_value(&box_val).v == 123, 0);   
    }
}

// Another module
module 0x9876dcda::test {
    use rooch_examples::box::{Self, Box};

    struct MyData has key, store, drop, copy {
        v: u64,
    }

    #[test]
    #[expected_failure]
    fun test() {
        let data = MyData{ v: 123 };
        let box_val = box::create_box<MyData, Box<MyData>>(data);
        assert!(box::box_value(&box_val).v == 123, 0);   
    }
}
