//# init --addresses creator=0x42

//# publish
module creator::test {
    use std::string::String;
    use moveos_std::object_id::ObjectID;

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
    use creator::test;

    fun main(_s: signer) {
        test::invoke_bar_bool();
        test::invoke_bar_u8();
        test::invoke_bar_u16();
        test::invoke_bar_u32();
        test::invoke_bar_u64();
        test::invoke_bar_u128();
        test::invoke_bar_u256();
        test::invoke_bar_address();
        test::invoke_bar_string();
        test::invoke_bar_object_id();
        test::invoke_bar_signer();
        test::invoke_bar_vector_bool();
        test::invoke_bar_vector_u8();
        test::invoke_bar_vector_u16();
        test::invoke_bar_vector_u32();
        test::invoke_bar_vector_u64();
        test::invoke_bar_vector_u128();
        test::invoke_bar_vector_u256();
        test::invoke_bar_vector_address();
        test::invoke_bar_vector_string();
        test::invoke_bar_vector_object_id();
        test::invoke_bar_vector_signer();
        test::invoke_bar_vector_StructT();
    }
}