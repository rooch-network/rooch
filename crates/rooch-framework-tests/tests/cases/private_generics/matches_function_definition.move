//# init --addresses creator=0x42

//# publish
module creator::test {
    #[private_generics(T1, T2, T3)]
    public fun create_box_one<T1, T2, T3>() {}

    #[private_generics(T1, T2)]
    public fun create_box_two<T1, T2, T3>() {}

    #[private_generics(T1)]
    public fun create_box_three<T1, T2, T3>() {}
}