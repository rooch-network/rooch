module basic_object::child_object{
    use std::string::String;
    use moveos_std::object::{Self, Object, ObjectID};

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

    public fun update_name(child: &mut Object<Child>, name: String){
        let child = object::borrow_mut(child);
        child.name = name;
    }

    public fun update_age(child: &mut Object<Child>, age: u64){
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

    #[test_only]
    public fun init_for_testing(){
        init();
    }

    #[test]
    fun test_child(){
        init();
        let child = new_child(std::string::utf8(b"Alice"));
        update_name(&mut child, std::string::utf8(b"Bob"));
        update_age(&mut child,11);
        assert!(get_age(&child) == 11, 1000);
        remove_child(child);
    }
}

module basic_object::third_party_module_for_child_object{
    use std::string::String;
    use moveos_std::tx_context;
    use moveos_std::object::{Self, Object, ObjectID};
    use basic_object::child_object::{Self, Child};

    public entry fun create_child(name: String){
        let child = child_object::new_child(name);
        object::transfer(child, tx_context::sender());
    }

    public entry fun update_child_name(child: &mut Object<Child>, name: String){
        child_object::update_name(child, name);
    }

    public entry fun update_child_name_via_id(sender: &signer, child_id: ObjectID, name: String){
        let child = object::borrow_mut_object<Child>(sender, child_id);
        child_object::update_name(child, name);
    }

    public entry fun update_child_age(child: &mut Object<Child>, age: u64){
        child_object::update_age(child, age);
    }

    public entry fun remove_child_via_id(sender: &signer, child_id: ObjectID){
        let child = object::take_object<Child>(sender, child_id);
        child_object::remove_child(child);
    }

    #[test]
    fun test_create_child(){
        child_object::init_for_testing();
        create_child(std::string::utf8(b"Alice"));
    }
}

