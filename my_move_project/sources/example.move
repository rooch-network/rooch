module my_move_project::counter {
    use moveos_std::object::{Self, Object};

    /// Resource that wraps an integer counter
    struct Counter has key {
        value: u64,
    }

    /// Constructor for Counter
    public fun create(_account: &signer): Object<Counter> {
        let counter = Counter { value: 0 };
        object::new(counter)
    }

    /// Increment the counter
    public fun increment(counter_obj: &mut Object<Counter>) {
        let counter = object::borrow_mut(counter_obj);
        counter.value = counter.value + 1;
    }

    /// Get the counter value
    public fun value(counter_obj: &Object<Counter>): u64 {
        let counter = object::borrow(counter_obj);
        counter.value
    }
}