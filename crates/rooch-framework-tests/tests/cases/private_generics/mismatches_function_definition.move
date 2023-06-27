//# init --addresses creator=0x42

//# publish
module creator::test {
    #[private_generics(T2)]
    public fun create_box<T1>() {}
}