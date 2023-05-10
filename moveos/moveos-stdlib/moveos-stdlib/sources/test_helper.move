#[test_only]
/// Module providing testing functionality. Only included for tests.
module moveos_std::test_helper {
    public native fun destroy<T>(x: T);
}
