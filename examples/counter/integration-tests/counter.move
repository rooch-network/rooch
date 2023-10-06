//# init --addresses genesis=0x1

//create account by bob self
//# run --signers genesis
script {
    use moveos_std::context::Context;
    use rooch_examples::counter;

    fun main(ctx: &mut Context, sender: &signer) {
        counter::init_for_test(ctx, sender);
    }
}