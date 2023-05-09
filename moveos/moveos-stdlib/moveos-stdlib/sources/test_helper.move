#[test_only]
/// Module providing testing functionality. Only included for tests.
module moveos_std::test_helper {
    friend moveos_std::storage_context;

    #[test_only]
    /// Testing only: allow to drop Move Value
    public(friend) fun drop_unchecked_move_value<MoveValue>(bytes: vector<u8>) {
        // object_storage::drop_object_storage(object_storage_mut(this));
        // _ = this;
        // let _ = this;
        // unit_test::drop_unchecked(bcs::to_bytes(&this));

        drop_unchecked<MoveValue>(bytes);
    }

    native fun drop_unchecked<MoveValue>(bytes: vector<u8>);
}
