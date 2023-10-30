// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

module rooch_framework::display{
    use std::string::String;
    use moveos_std::object::{Self, Object};
    use moveos_std::context::{Self, Context};
    use moveos_std::simple_map;

    /// Display<T> is a singleton object
    /// It is used to define the display of the `T`
    struct Display<phantom T> has key {
        sample_map: simple_map::SimpleMap<String, String>
    }

    #[private_generics(T)]
    /// Create a new Display object, Object<Display<T>> is a singleton object.
    /// Only the module of `T` can create a new Display object for `T`.
    /// The Display Object is permanent, can not be deleted.
    public fun new<T>(ctx: &mut Context): &mut Object<Display<T>> {
        let obj = context::new_singleton(ctx, Display<T> {
            sample_map: simple_map::create()
        });
        object::to_permanent(obj);
        context::borrow_mut_singleton<Display<T>>(ctx)
    }

    #[private_generics(T)]
    /// Borrow the mut Display object
    /// Only the module of `T` can borrow the mut Display object for `T`.
    public fun borrow_mut<T>(ctx: &mut Context): &mut Object<Display<T>> {
        context::borrow_mut_singleton<Display<T>>(ctx)
    }

    public fun set_value<T>(self: &mut Object<Display<T>>, key: String, value: String) {
        let display_ref = object::borrow_mut(self);
        simple_map::add(&mut display_ref.sample_map, key, value);
    }

    public fun borrow_value<T>(self: & Object<Display<T>> , key: &String): &String {
        let display_ref = object::borrow(self);
        simple_map::borrow(&display_ref.sample_map, key)
    }

    public fun borrow_mut_value<T>(self: &mut Object<Display<T>>, key: &String): &mut String {
        let display_ref = object::borrow_mut(self);
        simple_map::borrow_mut(&mut display_ref.sample_map, key)
    }

    public fun remove_value<T>(self: &mut Object<Display<T>>, key: &String) {
        let display_ref = object::borrow_mut(self);
        simple_map::remove(&mut display_ref.sample_map, key);
    }

    public fun keys<T>(self: & Object<Display<T>>): vector<String> {
        let display_ref = object::borrow(self);
        simple_map::keys(& display_ref.sample_map)
    }

    public fun values<T>(self: & Object<Display<T>>): vector<String> {
        let display_ref = object::borrow(self);
        simple_map::values(& display_ref.sample_map)
    }

    public fun contains_key<T>(self: & Object<Display<T>>, key: &String): bool {
        let display_ref = object::borrow(self);
        simple_map::contains_key(& display_ref.sample_map, key)
    }

}