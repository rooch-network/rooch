//# init --addresses creator=0x42

//# publish
module creator::test {
    struct NoAbilities {}

    #[private_generics(T)]
    public fun bar<T>() {}
}

//# run
script {
    use creator::test;

    fun main() {
        test::bar<NoAbilities>();
    }
}