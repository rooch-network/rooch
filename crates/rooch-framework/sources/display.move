module rooch_framework::display{
    use std::string::String;
    use moveos_std::object_ref;
    use moveos_std::context::Context;
    use moveos_std::context;
    use moveos_std::object_ref::Object;
    use moveos_std::simple_map;

    struct Display<phantom T> has key, store,drop,copy {
        sample_map: simple_map::SimpleMap<String, String>
    }

    public fun new<T>(ctx: &mut Context): Object<Display<T>> {
        context::new_singleton_object(ctx, Display<T> {
            sample_map: simple_map::create()
        })
    }

    public fun set<T>(self: &mut Object<Display<T>>, key: String, value: String) {
        let display_ref = object_ref::borrow_mut(self);
        simple_map::add(&mut display_ref.sample_map, key, value);
    }

    public fun borrow<T>(self: & Object<Display<T>> , key: &String): &String {
        let display_ref = object_ref::borrow(self);
        simple_map::borrow(&display_ref.sample_map, key)
    }

    public fun borrow_mut<T>(self: &mut Object<Display<T>>, key: &String): &mut String {
        let display_ref = object_ref::borrow_mut(self);
        simple_map::borrow_mut(&mut display_ref.sample_map, key)
    }

    public fun remove<T>(self: &mut Object<Display<T>>, key: &String) {
        let display_ref = object_ref::borrow_mut(self);
        simple_map::remove(&mut display_ref.sample_map, key);
    }

    public fun keys<T>(self: & Object<Display<T>>): vector<String> {
        let display_ref = object_ref::borrow(self);
        simple_map::keys(& display_ref.sample_map)
    }

    public fun values<T>(self: & Object<Display<T>>): vector<String> {
        let display_ref = object_ref::borrow(self);
        simple_map::values(& display_ref.sample_map)
    }

    public fun contains_key<T>(self: & Object<Display<T>>, key: &String): bool {
        let display_ref = object_ref::borrow(self);
        simple_map::contains_key(& display_ref.sample_map, key)
    }

}