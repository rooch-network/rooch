module rooch_framework::display{
    use std::ascii::String;
    use moveos_std::simple_map;

    struct Display has key, store,drop,copy {
        sample_map: simple_map::SimpleMap<String, String>
    }

    public fun new (): Display {
        Display {
            sample_map: simple_map::create()
        }
    }

    public fun set (self: &mut Display, key: String, value: String) {
        simple_map::add(&mut self.sample_map, key, value);
    }

    public fun borrow (self: & Display, key: String): &String {
        simple_map::borrow(&mut self.sample_map, &key)
    }

    public fun borrow_mut (self: &mut Display, key: String): &mut String {
        simple_map::borrow_mut(&mut self.sample_map, &key)
    }

    public fun remove (self: &mut Display, key: String) {
        simple_map::remove(&mut self.sample_map, &key);
    }

    public fun keys (self: & Display): vector<String> {
        simple_map::keys(& self.sample_map)
    }

    public fun values (self: & Display): vector<String> {
        simple_map::values(& self.sample_map)
    }

    public fun contains_key (self: & Display, key: String) -> bool {
        simple_map::contains(& self.sample_map, key)
    }
}