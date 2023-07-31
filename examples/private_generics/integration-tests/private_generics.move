//# init --addresses test=0x42

//# run --signers test
script {
    use rooch_examples::Test1;
    use rooch_examples::Test2;
    use rooch_examples::Test3;

    fun main() {
        Test1::test();
        Test2::run();
        Test3::run(); // this test will report error!
    }
}