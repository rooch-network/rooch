module app_admin::admin {

    use moveos_std::object::{Self,transfer};

    struct AdminCap has store, key {}

    fun init() {
        let admin_cap = object::new_named_object(AdminCap {});
        transfer(admin_cap, @app_admin)
    }

    #[deprecated]
    public entry fun fix_admin_cap() {
    }

    #[test_only]
    use moveos_std::object::Object;

    #[test_only]
    public fun init_for_test(): &mut Object<AdminCap>{
        init();
        let admin_cap_id = object::named_object_id<AdminCap>();
        object::borrow_mut_object_extend<AdminCap>(admin_cap_id)
    }
}