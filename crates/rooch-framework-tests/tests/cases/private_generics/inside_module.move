//# init --addresses creator=0x42

//# publish
module creator::test {
    struct Foo has key {
        x: u64,
    }

    #[private_generics(T)]
    fun publish_foo<T: key>(s: &signer) {
        move_to<Foo>(s, Foo { x: 500 })
    }

    public fun invoke_publish_foo(s: &signer) {
        publish_foo<Foo>(s);
    }
}

//# run --signers creator
script {
    use creator::test;

    fun main(s: signer) {
        test::invoke_publish_foo(&s);
    }
}
