//# init --addresses test=0x42 A=0x43

//# publish

module test::m {
    use mos_std::tx_context::{Self, TxContext};
    use mos_std::object::{Self, UID};

    struct S has store, key { id: UID }
    struct Cup<phantom T: store> has store, key { id: UID }

    public entry fun mint_s(ctx: &mut TxContext) {
        let id = object::new(ctx);
        object::transfer(S { id }, tx_context::sender(ctx))
    }

    public entry fun mint_cup<T: store>(ctx: &mut TxContext) {
        let id = object::new(ctx);
        object::transfer(Cup<T> { id }, tx_context::sender(ctx))
    }
}

// Mint S to A. Transfer S from A to B

//# run test::m::mint_s --signers A

// Mint Cup<S> to A. Transfer Cup<S> from A to B

//# run test::m::mint_cup --type-args test::m::S --signers A

