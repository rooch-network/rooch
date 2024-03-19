//# init --addresses creator=0x42

//# publish
module creator::foo {
    struct T has drop {
        x: u64,
    }

    #[private_generics(T)]
    public fun empty_foo<T>() {}
}

//# publish
module creator::bar {
    use std::string::String;
    use moveos_std::object::ObjectID;

    struct StructT has copy, drop {
        x: u64,
    }

    #[private_generics(T)]
    fun bar<T>() {}

    public fun invoke_bar_bool() {
        bar<bool>();
    }

    public fun invoke_bar_u8() {
        bar<u8>();
    }

    public fun invoke_bar_u16() {
        bar<u16>();
    }

    public fun invoke_bar_u32() {
        bar<u32>();
    }

    public fun invoke_bar_u64() {
        bar<u64>();
    }

    public fun invoke_bar_u128() {
        bar<u128>();
    }

    public fun invoke_bar_u256() {
        bar<u256>();
    }

    public fun invoke_bar_address() {
        bar<address>();
    }

    public fun invoke_bar_string() {
        bar<String>();
    }

    public fun invoke_bar_object_id() {
        bar<ObjectID>();
    }

    public fun invoke_bar_signer() {
        bar<signer>();
    }
    
    public fun invoke_bar_vector_bool() {
        bar<vector<bool>>();
    }
    
    public fun invoke_bar_vector_u8() {
        bar<vector<u8>>();
    }
    
    public fun invoke_bar_vector_u16() {
        bar<vector<u16>>();
    }
    
    public fun invoke_bar_vector_u32() {
        bar<vector<u32>>();
    }
    
    public fun invoke_bar_vector_u64() {
        bar<vector<u64>>();
    }
    
    public fun invoke_bar_vector_u128() {
        bar<vector<u128>>();
    }

    public fun invoke_bar_vector_u256() {
        bar<vector<u256>>();
    }

    public fun invoke_bar_vector_address() {
        bar<vector<address>>();
    }

    public fun invoke_bar_vector_string() {
        bar<vector<String>>();
    }

    public fun invoke_bar_vector_object_id() {
        bar<vector<ObjectID>>();
    }
    
    public fun invoke_bar_vector_signer() {
        bar<vector<signer>>();
    }
    
    public fun invoke_bar_vector_StructT() {
        bar<vector<StructT>>();
    }
}

//# run --signers creator
script {
    // creator::bar doesn't exist due to publishing failure
    // creator::foo exists
    use creator::foo::{Self, T};
    use std::string::String;
    use moveos_std::object::ObjectID;

    fun main(_s: signer) {
        foo::empty_foo<bool>();
        foo::empty_foo<u8>();
        foo::empty_foo<u16>();
        foo::empty_foo<u32>();
        foo::empty_foo<u64>();
        foo::empty_foo<u128>();
        foo::empty_foo<u256>();
        foo::empty_foo<address>();
        foo::empty_foo<String>();
        foo::empty_foo<ObjectID>();
        foo::empty_foo<signer>();
        foo::empty_foo<vector<bool>>();
        foo::empty_foo<vector<u8>>();
        foo::empty_foo<vector<u16>>();
        foo::empty_foo<vector<u32>>();
        foo::empty_foo<vector<u64>>();
        foo::empty_foo<vector<u128>>();
        foo::empty_foo<vector<u256>>();
        foo::empty_foo<vector<address>>();
        foo::empty_foo<vector<String>>();
        foo::empty_foo<vector<ObjectID>>();
        foo::empty_foo<vector<signer>>();
        foo::empty_foo<vector<T>>();
    }
}