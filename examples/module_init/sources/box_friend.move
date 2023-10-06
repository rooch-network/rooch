module rooch_examples::box_friend {
    use std::string::{Self, String};
    use rooch_examples::box;
    use moveos_std::object::Object;
    use rooch_examples::box::Box;
    use moveos_std::context::Context;
    use std::debug;

    friend rooch_examples::box_fun;

    // for test
    fun init(_ctx: &mut Context) {
        debug::print<String>(&string::utf8(b"module box_friend init finish"));
    }

    public(friend) fun change_box(obj: Object<Box>): Object<Box> {
        string::append(&mut box::name(&obj), string::utf8(b"z"));
        // box::set_name(&mut obj, box::name(&obj));

        let count = box::count(&obj) + 1;
        box::set_count(&mut obj, count);
        obj
    }
}
