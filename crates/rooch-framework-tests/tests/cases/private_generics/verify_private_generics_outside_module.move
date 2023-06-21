//# init --addresses creator=0x42 A=0x43

//# publish
module creator::test {
    struct Foo has key, drop {
        x: u64,
    }

    #[private_generics(T)]
    fun publish_foo<T: key>(s: &signer) {
        move_to<Foo>(s, Foo { x: 500 })
    }
}

//# publish
module A::m {
    use creator::test::Foo;

    struct Bar has key, drop {
        v: u128,
    }

    #[private_generics(V)]
    fun publish_bar<V: drop>(s: &signer) {
        move_to<Bar>(s, Bar { v: 100 })
    }

    public fun invoke_publish_bar(s: &signer) {
        publish_bar<Foo>(s);
    }
}

//# run --signers creator
script {
    use A::m;

    fun main(s: signer) {
        m::invoke_publish_bar(&s);
    }
}