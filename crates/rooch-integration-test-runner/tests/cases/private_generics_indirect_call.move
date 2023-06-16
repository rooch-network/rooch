//# init --addresses creator=0x42

//# publish
module creator::test {
    struct Foo has key {
        x: u64,
    }

    #[private(T1)]
    public fun publish_foo<T1>(s: &signer) {
        move_to<Foo>(s, Foo { x: 500 })
    }

    public fun run(s: &signer) {
        publish_foo<u64>(s)
    }
}

//# run --signers creator
script {
    use creator::test;

    fun main(s: signer) {
        test::run(&s);
    }
}