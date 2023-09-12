module simple_object::simple_object {
    use std::string;
    use std::string::String;
    use moveos_std::account_storage;
    use moveos_std::signer;
    use moveos_std::object_storage;
    use moveos_std::storage_context::{Self, StorageContext};
    use moveos_std::object_id::ObjectID;
    use moveos_std::object::{Self, Object};

    struct Student has key {
        name: String,
        age: u32,
    }

    fun create_student(
        sctx: &mut StorageContext,
        owner: &signer,
        name: String,
        age: u32,
    ): ObjectID {
        let tx_ctx = storage_context::tx_context_mut(sctx);
        let owner_addr = signer::address_of(owner);
        let student = Student { name, age };
        let student_obj = object::new(tx_ctx, owner_addr, student);
        let obj_id = object::id(&student_obj);
        let obj_storage = storage_context::object_storage_mut(sctx);

        object_storage::add(obj_storage, student_obj);
        obj_id
    }

    fun id(obj: &Object<Student>): ObjectID {
        object::id(obj)
    }

    fun name(obj: &Object<Student>): String {
        object::borrow(obj).name
    }

    fun age(obj: &Object<Student>): u32 {
        object::borrow(obj).age
    }

    fun set_age(obj: &mut Object<Student>, age: u32) {
        object::borrow_mut(obj).age = age;
    }

    fun get_student(sctx: &StorageContext, obj_id: ObjectID): &Object<Student> {
        let obj_storage = storage_context::object_storage(sctx);
        object_storage::borrow(obj_storage, obj_id)
    }

    entry fun create_student_entry(sctx: &mut StorageContext, owner: &signer) {
        let student = Student {
            name: string::utf8(b"alice"),
            age: 10
        };
        account_storage::global_move_to(sctx, owner, student);
    }

    entry fun set_student_name_entry(sctx: &mut StorageContext, owner: &signer, stu_name: String) {
        let owner_addr = signer::address_of(owner);
        let student = account_storage::global_borrow_mut<Student>(sctx, owner_addr);
        student.name = stu_name;
    }

    entry fun set_student_age_entry(sctx: &mut StorageContext, owner: &signer, stu_age: u32) {
        let owner_addr = signer::address_of(owner);
        let student = account_storage::global_borrow_mut<Student>(sctx, owner_addr);
        student.age = stu_age;
    }
}