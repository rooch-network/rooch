//# init --addresses test=0x42

//# run --signers test
script {
    use rooch_examples::Test1;

    fun main() {
        Test1::test();
    }
}

//# run --signers test
script {
    use rooch_examples::Test2;

    fun main() {
        Test2::run();
    }
}

//# run --signers test
script {
    use rooch_examples::Test3;

    fun main() {
        // FIXME: expected failure
        Test3::run();
    }
}