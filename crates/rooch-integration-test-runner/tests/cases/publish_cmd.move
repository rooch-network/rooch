//# init --addresses creator=0x42

//# publish
module creator::test {
    
    use moveos_std::account;

    struct Foo has key, store {
        x: u64,
    }

    public fun publish_foo(s: &signer) {
        account::move_resource_to<Foo>(s, Foo { x: 500 })
    }
}

//# run --signers creator
script {
    use creator::test;
    

    fun main(s: signer) {
        test::publish_foo(&s);
    }
}

//# view
//#     --address 0x42
//#     --resource 0x42::test::Foo