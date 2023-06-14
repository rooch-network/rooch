//# init --addresses creator=0x42

//# publish
module creator::test {
    struct StructT has copy, drop {
        x: u64,
    }

    #[private_generics(T)]
    public fun bar<T>() {}
}

//# run
script {
    use creator::test;

    fun main() {
        test::bar<bool>();
        test::bar<u8>();
        test::bar<u16>();
        test::bar<u32>();
        test::bar<u64>();
        test::bar<u128>();
        test::bar<u256>();
        test::bar<address>();
        test::bar<signer>();
        test::bar<test::StructT>();
        test::bar<vector<bool>>();
        test::bar<vector<u8>>();
        test::bar<vector<u16>>();
        test::bar<vector<u32>>();
        test::bar<vector<u64>>();
        test::bar<vector<u128>>();
        test::bar<vector<u256>>();
        test::bar<vector<address>>();
        test::bar<vector<signer>>();
        test::bar<vector<test::StructT>>();
    }
}