//# init --addresses creator=0x42 A=0x43

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

//# publish
module A::m {
    struct Bar has key, drop {
        v: u128,
    }

    #[private_generics(V)]
    public fun publish_bar<V: drop>(s: &signer) {
        move_to<Bar>(s, Bar { v: 100 })
    }
}

//# run --signers creator
script {
    use creator::test;
    use A::m;

    fun main(s: signer) {
        // success, Bar has key, drop // TBD: has no errors regarding private_generics
        test::publish_foo<m::Bar>(&s);
    }
}