//# init --addresses test=0x42

//# run --signers test
script {
    use rooch_examples::Data;

    fun main() {
        Data::run();
    }
}