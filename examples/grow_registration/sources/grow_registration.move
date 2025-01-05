module grow_registration::grow_registration {

    use std::signer::address_of;
    use std::string::String;
    use std::vector;
    use std::vector::length;
    use grow_bitcoin::grow_point_v3::{PointBox, timestamp, value};
    use app_admin::admin::AdminCap;
    use moveos_std::object;
    use moveos_std::table;
    use moveos_std::object::{ObjectID, to_shared, Object};
    use moveos_std::table::Table;


    const ErrorInvalidVoteTime: u64 = 1;

    struct Registration has key, store{
        project_id: String,
        register_point_box: Table<ObjectID, bool>,
        user_info: Table<address, UserInfo>,
        start_time: u64,
        end_time: u64,
    }

    struct UserInfo has store, drop{
        register_info: String,
        amount: u256
    }

    public entry fun create_registration(project_id: String, start_time: u64, end_time: u64, _admin: &mut Object<AdminCap>) {
        let registration = Registration {
            project_id,
            register_point_box: table::new(),
            user_info: table::new(),
            start_time,
            end_time
        };
        to_shared(object::new_with_id(project_id, registration));
    }

    public entry fun register_batch(signer: &signer, registration_obj: &mut Object<Registration>, point_box_objs: vector<ObjectID>, register_info: String) {
        let i = 0;
        while (i < length(&point_box_objs)) {
            let point_box_id = *vector::borrow(&point_box_objs, i);
            let point_box = object::borrow_mut_object<PointBox>(signer, point_box_id);
            register(signer, registration_obj, point_box, register_info);
            i = i + 1;
        }
    }

    public entry fun register(signer: &signer, registration_obj: &mut Object<Registration>, point_box_obj: &mut Object<PointBox>, register_info: String) {
        let registration = object::borrow_mut(registration_obj);
        let vote_time = timestamp(point_box_obj);
        assert!(vote_time >= registration.start_time && vote_time <= registration.end_time, ErrorInvalidVoteTime);
        let user_info = table::borrow_mut_with_default(&mut registration.user_info, address_of(signer), UserInfo{
            register_info,
            amount: 0
        });
        user_info.amount = user_info.amount + value(point_box_obj);
        table::add(&mut registration.register_point_box, object::id(point_box_obj), true);
    }

    public entry fun update_register_info(signer: &signer, registration_obj: &mut Object<Registration>, register_info: String) {
        let registration = object::borrow_mut(registration_obj);
        let user_info = table::borrow_mut(&mut registration.user_info, address_of(signer));
        user_info.register_info = register_info
    }
}
