//# init --addresses creator=0x42

//# publish
module creator::test {
    struct Foo has key {
        x: u64,
    }

    #[private_generics(T)]
    public fun publish_foo<T: key>(s: &signer) {
        move_to<Foo>(s, Foo { x: 500 })
    }
}

//# run --signers creator
script {
    use creator::test;

    fun main(s: signer) {
        test::publish_foo<test::Foo>(&s);
    }
}

//# view
//#     --address 0x42
//#     --resource 0x42::test::Foo
