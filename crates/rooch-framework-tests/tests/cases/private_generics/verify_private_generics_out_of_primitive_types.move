//# init --addresses creator=0x42

//# publish
module creator::test {
    struct NoAbilities {}

    #[private_generics(T)]
    fun bar<T>() {}

    public fun invoke_bar() {
        bar<NoAbilities>();
    }
}

//# run --signers creator
script {
    use creator::test;

    fun main(_s: signer) {
        test::invoke_bar();
    }
}