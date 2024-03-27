//# init --addresses test=0x42 A=0x43

//# publish

module test::child_object{
    use std::string::String;
    use moveos_std::object::{Self, Object, ObjectID};
    use moveos_std::tx_context;

    struct Parent has key{
        sequencer: u64,
    }
    struct Child has key,store{
        sequencer: u64,
        name: String,
    }

    struct NewChildEvent has copy, drop{
        id: ObjectID,
        sequencer: u64,
    }

    struct ChildRemovedEvent has copy, drop{
        id: ObjectID,
        sequencer: u64,
    }

    fun init(){
        let parent = object::new_named_object(Parent{sequencer:0});
        object::to_shared(parent);
    }

    fun borrow_mut_parent() : &mut Object<Parent> {
        let parent_obj_id = object::named_object_id<Parent>();
        object::borrow_mut_object_shared<Parent>(parent_obj_id)
    }

    public fun new_child(name: String): Object<Child> {
        let parent_obj = borrow_mut_parent();
        let new_sequencer = object::borrow(parent_obj).sequencer + 1;
        let child = object::add_object_field(parent_obj, Child{sequencer:new_sequencer, name:name});
        let id = object::id(&child);
        std::debug::print(&id);
        object::borrow_mut(parent_obj).sequencer = new_sequencer;
        moveos_std::event::emit(NewChildEvent{id:id, sequencer:new_sequencer});
        child
    }

    public fun remove_child(child: Object<Child>){
        remove_child_property<u64>(&mut child, std::string::utf8(b"age"));
        let parent_obj = borrow_mut_parent();
        let id = object::id(&child);
        let Child{ sequencer, name:_ } = object::remove_object_field(parent_obj, child);
        moveos_std::event::emit(ChildRemovedEvent{id:id, sequencer:sequencer});
    }

    public entry fun update_name(child: &mut Object<Child>, name: String){
        let child = object::borrow_mut(child);
        child.name = name;
    }

    public entry fun update_age(child: &mut Object<Child>, age: u64){
        update_child_property(child, std::string::utf8(b"age"), age);
    }

    public fun get_age(child: &Object<Child>) : u64{
        *get_child_property(child, std::string::utf8(b"age"))
    }

    fun update_child_property<V: store + drop>(child: &mut Object<Child>, property: String, value: V){
        object::upsert_field(child, property, value);
    }

    fun get_child_property<V: store + drop>(child: &Object<Child>, property: String): &V{
        object::borrow_field(child, property)
    }

    fun remove_child_property<V: store + drop>(child: &mut Object<Child>, property: String){
        if(object::contains_field(child, property)){
            let _v:V = object::remove_field(child, property);
        }
    }

    public entry fun create_child(name: String){
        let child = new_child(name);
        object::transfer(child, tx_context::sender());
    }

    public entry fun remove_child_via_id(sender: &signer, child_id: ObjectID){
        let child = object::take_object<Child>(sender, child_id);
        remove_child(child);
    }

    public entry fun check_age(child: &Object<Child>, age: u64){
        assert!(object::contains_field(child, std::string::utf8(b"age")), 76);
        assert!(get_age(child) == age, 77);
    }

}

//# run test::child_object::create_child --signers A --args string:alice

//# run test::child_object::update_age --signers A --args object:0x5370106f3bd3bf65af644c293d9568d8e615d589bc5d02c0188276df5af07dbb223858a43c3db3880a0f64ad2c25194ee770bd93f1e6cbceb7de952c2aac0d3c u64:42

//# run test::child_object::check_age --signers A --args object:0x5370106f3bd3bf65af644c293d9568d8e615d589bc5d02c0188276df5af07dbb223858a43c3db3880a0f64ad2c25194ee770bd93f1e6cbceb7de952c2aac0d3c u64:42