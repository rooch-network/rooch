// module rooch_examples
module rooch_examples::module1 {
    // Define a Data type
    struct Data has copy, drop {
        v: u64
    }

    // Define a Box type
    struct Box<T> has drop {
        v: T
    }

    // Create an instance of type Data
    fun new_data(value: u64): Data {
        Data { v: value }
    }

    // Create an instance of type Box<T>
    #[private_generics(T1, T2)]
    public fun new_box<T1, T2, T3>(value: T1): Box<T1> {
        Box { v: value }
    }

    // Get the value inside the Box<T>
    public fun get_box_value<T: copy>(box: &Box<T>): T {
        box.v
    }

    // Unit Test:
    #[test]
    // #[expected_failure]
    fun test1() {
        let data = new_data(123);
        let box = new_box<Data, Data, Data>(data);
        assert!(get_box_value<Data>(&box).v == 123, 1000);
        // assert!(get_box_value(&box) == Data{v:1234}, 10001)
    }
}

module rooch_examples::module2 {
    use rooch_examples::module1::{new_box, get_box_value, Data};

    struct Data2 has copy, drop {
        v: u64
    }

    struct Data3 has copy, drop {
        v: u32
    }

    #[test]
    #[expected_failure]
    fun test2() {
        let data2 = Data2 { v: 789 };
        let box = new_box<Data2, Data3, u64>(data2);
        assert!(get_box_value(&box).v == 789, 2000);
    }
}