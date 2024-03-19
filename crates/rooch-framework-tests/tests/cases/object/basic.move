//# init --addresses test=0x42 A=0x43

//# publish

module test::m {
    
    use moveos_std::object::{Self, Object};
    use moveos_std::object::ObjectID;
    use std::debug;

    struct S has store, key { v: u8 }

    public entry fun mint() {
        let tx_hash = moveos_std::tx_context::tx_hash();
        debug::print(&tx_hash);
        // if the tx hash change, need to figure out why.
        assert!(x"d9ee14951f05eafce05da16395f3acd8324708a3b608ebf13fb41ffcbef87e30" == tx_hash, 1000);
        let obj = object::new(S { v: 1});
        debug::print(&obj);
        object::transfer(obj, moveos_std::tx_context::sender());
    }

    public entry fun update(obj_s: &mut Object<S>){
        let s = object::borrow_mut(obj_s);
        s.v = 2;
    }

    //We can not use `Object<S>` as transaction argument now, so use ObjectID
    public entry fun remove(sender: &signer, obj_s_id: ObjectID) {
        let obj_s = object::take_object<S>(sender, obj_s_id);
        let S{ v } = object::remove(obj_s);
        assert!(v == 2, 1001);
    }

}

// Mint
//# run test::m::mint --signers A

//# view_object --object-id 0x8f684aa792b9b1058aeccd3941849e9662132d81c974b826a9c6bddae8880bd6

//Update
//# run test::m::update --signers A --args object:0x8f684aa792b9b1058aeccd3941849e9662132d81c974b826a9c6bddae8880bd6

//# view_object --object-id 0x8f684aa792b9b1058aeccd3941849e9662132d81c974b826a9c6bddae8880bd6

//Remove
//# run test::m::remove --signers A --args object_id:0x8f684aa792b9b1058aeccd3941849e9662132d81c974b826a9c6bddae8880bd6

// Check if removed
//# view_object --object-id 0x8f684aa792b9b1058aeccd3941849e9662132d81c974b826a9c6bddae8880bd6
