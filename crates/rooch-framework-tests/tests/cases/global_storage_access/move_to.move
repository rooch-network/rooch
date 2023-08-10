//# init --addresses creator=0x42 A=0x43

//# publish
module creator::test {
    struct Foo has key, drop {
        x: u64,
    }

    public fun publish_foo<T: key>(s: &signer) {
	    move_to<Foo>(s, Foo { x: 500 })
    }
}

//# run --signers creator
script {
    use creator::test::{Self, Foo};

    fun main(s: signer) {
        test::publish_foo<Foo>(&s);
    }
}
