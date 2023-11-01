// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

module moveos_std::display{
    use std::string::String;
    use moveos_std::object::{Self, Object};
    use moveos_std::context::{Self, Context};
    use moveos_std::simple_map;

    /// Display<T> is a singleton object
    /// It is used to define the display of the `T`
    /// The Display Object is permanent, can not be deleted after created.
    struct Display<phantom T> has key {
        sample_map: simple_map::SimpleMap<String, String>
    }

    #[private_generics(T)]
    /// Create or borrow_mut Display object for resource `T`
    /// Only the module of `T` can call this function.
    public fun resource_display<T: key>(ctx: &mut Context): &mut Object<Display<T>> {
        let object_id = object::singleton_object_id<Display<T>>();
        if (context::exist_object<Display<T>>(ctx, object_id)) {
            context::borrow_mut_object_extend<Display<T>>(ctx, object_id)
        }else{
            context::new_singleton(ctx, Display<T> {
                sample_map: simple_map::create()
            })
        }
    }

    #[private_generics(T)]
    /// Create or borrow_mut Display object for `Object<T>`
    /// Only the module of `T` can call this function.
    public fun object_display<T: key>(ctx: &mut Context): &mut Object<Display<Object<T>>> {
        let object_id = object::singleton_object_id<Display<Object<T>>>();
        if (context::exist_object<Display<Object<T>>>(ctx, object_id)) {
            context::borrow_mut_object_extend<Display<Object<T>>>(ctx, object_id)
        }else{
            context::new_singleton(ctx, Display<Object<T>> {
                sample_map: simple_map::create()
            })
        }
    }

    /// Set the key-value pair for the display object
    /// If the key already exists, the value will be updated, otherwise a new key-value pair will be created.
    public fun set_value<T>(self: &mut Object<Display<T>>, key: String, value: String) {
        let display_ref = object::borrow_mut(self);
        simple_map::upsert(&mut display_ref.sample_map, key, value);
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