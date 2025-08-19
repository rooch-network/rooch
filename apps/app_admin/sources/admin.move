module app_admin::admin {

    use moveos_std::object::{Self,transfer};
    use moveos_std::module_store;
    use moveos_std::signer;

    const ErrorNotUpgradePermission: u64 = 1;

    struct AdminCap has store, key {}

    fun init() {
        let admin_cap = object::new_named_object(AdminCap {});
        transfer(admin_cap, @app_admin)
    }

    #[deprecated]
    public entry fun fix_admin_cap() {
    }

    /// Transfer admin cap to a new address
    /// The sender must have upgrade permission to the app_admin package.
    public entry fun transfer_admin_cap(sender: &signer, to: address) {
        let sender_address = signer::address_of(sender);
        let has_permission = module_store::has_upgrade_permission(@app_admin, sender_address);
        assert!(has_permission, ErrorNotUpgradePermission);
        let admin_cap_id = object::named_object_id<AdminCap>();
        let admin_cap = object::take_object_extend<AdminCap>(admin_cap_id);
        object::transfer(admin_cap, to);
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