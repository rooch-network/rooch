// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

/// Example to show how to use moveos_std::display to setting
/// display templates for objects and resources.
module display::display{
    use std::string::{Self, String};
    use moveos_std::display;
    
    use moveos_std::object;
    use moveos_std::account;

    struct ResourceType has key{
        name: String,
        creator: address,
        description: String,
    }

    struct ObjectType has key, store {
        name: String,
        creator: address,
        description: String,
    }

    fun init(){
        // Template syntax

        // - `{var_name}`, no space between `{` with `var_name` and `var_name` with `}`.
        // - There are two types of template: 
        //     - for object meta fields. Availabel fields: {id}, {owner}, {flag}, {state_root}, {size}
        //     - for object instance fields, which depend on your object defination. 
        //       In this example, there are 3 fields: `name`, `creator` and `description` in a `ObjectType` object,
        //       so you can use templates: {value.name}, {value.creator}, {value.description} respectively.
        // - Supported type for object instance fields: primitive types, `0x1::string::String`, `0x2::object_id::ObjectID`. Other custom Move structs are not supported.      
        let display_obj = display::object_display<ObjectType>(); 
        display::set_value(display_obj, string::utf8(b"name"), string::utf8(b"{value.name}"));
        display::set_value(display_obj, string::utf8(b"uri"), string::utf8(b"https:://{owner}/{id}"));
        display::set_value(display_obj, string::utf8(b"description"), string::utf8(b"{value.description}"));
        display::set_value(display_obj, string::utf8(b"creator"), string::utf8(b"{value.creator}"));

        // For resource display templates:
        // - there are no object meta fields. {id}, {owner}, {flag}, {state_root}, {size} are not available.
        // - prefix `value.` is no need any more.
        let display_resource = display::resource_display<ResourceType>();
        display::set_value(display_resource, string::utf8(b"name"), string::utf8(b"{name}"));
        display::set_value(display_resource, string::utf8(b"description"), string::utf8(b"{description}"));
        display::set_value(display_resource, string::utf8(b"creator"), string::utf8(b"{creator}"));
    }

    /// Create a new ObjectType
    public entry fun create_object(
        
        name: String,
        creator: address,
        description: String,
    ) {

        let obj_type = ObjectType {
            name,
            creator,
            description,
        };

        let obj = object::new(
            
            obj_type
        );

        let sender = moveos_std::tx_context::sender();
        object::transfer(obj, sender);
    }

    /// Create a new ResourceType
    public entry fun create_resource(
        
        sender: &signer,
        name: String,
        creator: address,
        description: String,
    ) {

        let resource = ResourceType {
            name,
            creator,
            description,
        };

        account::move_resource_to(sender, resource);
    }
}
