module rooch_examples::counter {

     struct Counter has key, store {
        value:u64,
     }

     public fun init_(account: &signer){
        move_to(account, Counter{value:0});
     }

     public fun increase_() acquires Counter {
        let counter = borrow_global_mut<Counter>(@rooch_examples);
        counter.value = counter.value + 1;
     }

     public entry fun init(account: signer){
        Self::init_(&account)
     }

     public entry fun increase()  acquires Counter {
        Self::increase_()
     }

     public fun value(): u64 acquires Counter {
        let counter = borrow_global<Counter>(@rooch_examples);
        counter.value
     }
}