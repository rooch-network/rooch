module basic_object::pub_transfer{
    use moveos_std::object::{Self, Object};

    /// A object are transferable by anyone using `object::tansfer`
    struct PubTransfer has key,store{
        value: u64,
    }

    public fun new(value: u64) : Object<PubTransfer>{
        object::new(PubTransfer{value}) 
    }

}

module basic_object::custom_transfer{
    use moveos_std::object::{Self, Object};

    const ErrorInvalidTransfer: u64 = 1;

    /// A object support custom transfer rule
    struct CustomTransfer has key{
        value: u64,
    }

    public fun new(value: u64) : Object<CustomTransfer>{
        object::new(CustomTransfer{value}) 
    }

    fun custom_transfer_role(obj: &Object<CustomTransfer>) {
        assert!(object::borrow(obj).value > 10, ErrorInvalidTransfer);
    }

    public fun transfer(obj: Object<CustomTransfer>, new_owner: address) {
        custom_transfer_role(&obj);
        //We use `object::transfer_extend` to extend the transfer rule
        object::transfer_extend(obj, new_owner);
    }
}

module basic_object::third_party_module{
    use moveos_std::object;
    use moveos_std::tx_context;

    public fun create_and_pub_transfer(value: u64){
        let obj = basic_object::pub_transfer::new(value);
        object::transfer(obj, tx_context::sender());
    }

    public fun create_and_custom_transfer(value: u64){
        let obj = basic_object::custom_transfer::new(value);
        //We can not use `object::transfer` here, because the `T` of `object::transfer<T: key+store>` require the object to be `store`
        //And we also can not use `object::transfer_extend` here, because the `T` of `object::transfer_extend<T: key>` require `#[private_generics(T)]`
        //We only can use `object::transfer_extend` in the module where the object is defined
        //object::transfer(obj, tx_context::sender());
        basic_object::custom_transfer::transfer(obj, tx_context::sender());
    }

    #[test]
    fun test_transfer_success(){
        create_and_pub_transfer(100);
        create_and_custom_transfer(100);
    }

    #[test]
    #[expected_failure(abort_code = basic_object::custom_transfer::ErrorInvalidTransfer, location = basic_object::custom_transfer)]
    fun test_transfer_fail(){
        create_and_custom_transfer(5);
    }
}