module simple_object::simple_object {
    use std::string::String;
    use moveos_std::storage_context;
    use moveos_std::object_storage::ObjectStorage;
    use moveos_std::storage_context::{StorageContext, tx_context};
    use moveos_std::object_id::ObjectID;
    use moveos_std::object::{Self, Object};
    use moveos_std::table::Table;

    struct Student {
        name: String,
        age: u32,
        stu_id: u128,
        transcript: Table<u64, String>
    }

    struct StudentObjectCreated {
        obj_id: ObjectID,
        name: String,
        age: u32,
        stu_id: u128
    }

    struct Score<K, V> {
        key: K,
        value: V
    }

    struct Transcript {
        item: Score<String, u32>
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

    fun stu_id(obj: &Object<Student>): u128 {
        object::borrow(obj).stu_id
    }

    fun set_age(obj: &mut Object<Student>, age: u32) {
        object::borrow_mut(obj).age = age;
    }

    // fun create_student(
    //     storage_ctx: &mut StorageContext,
    //     name: String,
    //     age: u32,
    //     stu_id: u128
    // ): Object<Student> {
    //     let stu_info = {name, age, stu_id, };
    //     let tx_ctx = storage_context::tx_context_mut(storage_ctx);
    //     let owner = storage_context::sender(storage_ctx);
    //     let student = object::new(tx_ctx, owner, stu_info);
    //     student
    // }

}
