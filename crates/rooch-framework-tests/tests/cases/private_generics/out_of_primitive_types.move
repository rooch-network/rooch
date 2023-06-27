//# init --addresses creator=0x42

//# publish
module creator::test {
    struct Struct {
        x: u64
    }

    struct Recursive {
        y: Struct
    }

    #[private_generics(T)]
    fun bar<T>() {}

    public fun invoke_bar_struct() {
        bar<Struct>();
    }

    public fun invoke_bar_recursive() {
        bar<Recursive>();
    }
}

//# run --signers creator
script {
    use creator::test;

    fun main(_s: signer) {
        test::invoke_bar_struct();
        test::invoke_bar_recursive();
    }
}