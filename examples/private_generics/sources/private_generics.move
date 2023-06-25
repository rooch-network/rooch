module rooch_examples::Test1 {
    struct Box<T: store> has key, store, drop {
        value: T
    }

    struct Data has key, store, drop, copy {
        v: u64,
    }

    public fun new_data(v: u64): Data {
        Data{v: v}
    }

    #[private_generics(T1, T2)]
    public fun create_box<T1: store, T2, T3>(value: T1): Box<T1> {
        Box<T1> { value: value }
    }

    public fun box_value<T: copy+store>(box: &Box<T>): T {
        *&box.value
    }

    #[test]
    public fun test() {
        let data = Data{ v: 123 };

        let box_val = create_box<Data, Data, Box<u32>>(data);
        let _ = box_value(&box_val);
    }
}

module rooch_examples::Test2 {
    use rooch_examples::Test1::Box;
    use rooch_examples::Test1::create_box;
    use rooch_examples::Test1::box_value;

    struct InnerData has key, store, drop, copy {
        v: u64,
    }

    #[test]
    public fun run() {
        let data = InnerData { v: 789 };

        // Here is correct because the InnerData type is defined within the current module.
        let box_val = create_box<InnerData, InnerData, Box<u32>>(data);

        let _ = box_value(&box_val);
    }
}

module rooch_examples::Test3 {
    use rooch_examples::Test1::Data;
    use rooch_examples::Test1::Box;
    use rooch_examples::Test1::new_data;
    use rooch_examples::Test1::create_box;
    use rooch_examples::Test1::box_value;

    public fun run() {
        let data = new_data(789);

        // Here, it will report that `Data` is not defined within the current module,
        // because `Data` is imported from the `Test1` module.
        let box_val = create_box<Data, Data, Box<u32>>(data);

        let _ = box_value(&box_val);
    }
}