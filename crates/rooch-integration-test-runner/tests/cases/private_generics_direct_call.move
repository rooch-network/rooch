//# init --addresses creator=0x42

//# publish
module creator::test {
    use std::string;
    
    use moveos_std::account;

    struct Foo has key {
        x: u64,
    }

    #[private_generics(T1)]
    fun publish_foo<T1>(s: &signer) {
        account::move_resource_to<Foo>(s, Foo { x: 500 })
    }

    public fun run(s: &signer) {
        let _ = string::utf8(b"resource_object");
        publish_foo<Foo>(s)
    }
}

//# run --signers creator
script {
    use creator::test;
    

    fun main(s: &signer) {
        test::run(s);
    }
}

//# view
//#     --address 0x42
//#     --resource 0x42::test::Foo
