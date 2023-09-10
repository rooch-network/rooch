module simple_object::simple_object {
    use std::string;
    use std::string::String;
    use moveos_std::account_storage;
    use moveos_std::signer;
    use moveos_std::object_storage;
    use moveos_std::event;
    use moveos_std::storage_context::{Self, StorageContext};
    use moveos_std::object_id::ObjectID;
    use moveos_std::object::{Self, Object};

    struct Student has key {
        name: String,
        age: u32,
    }

    struct StudentCreatedEvent has store {
        obj_id: ObjectID,
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
        let student_create_event = StudentCreatedEvent { obj_id };
        let obj_storage = storage_context::object_storage_mut(sctx);

        object_storage::add(obj_storage, student_obj);
        event::emit(sctx, student_create_event);
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
            name: string::utf8(b"xxx"),
            age: 10
        };
        account_storage::global_move_to(sctx, owner, student);
    }

    entry fun set_student_name(sctx: &mut StorageContext, owner: &signer, stu_name: String) {
        let owner_addr = signer::address_of(owner);
        let student = account_storage::global_borrow_mut<Student>(sctx, owner_addr);
        student.name = stu_name;
    }

    entry fun set_student_age(sctx: &mut StorageContext, owner: &signer, stu_age: u32) {
        let owner_addr = signer::address_of(owner);
        let student = account_storage::global_borrow_mut<Student>(sctx, owner_addr);
        student.age = stu_age;
    }

    // entry fun remove_student(sctx: &mut StorageContext, owner: &signer, obj_id: ObjectID) {
    //     let owner_addr = signer::address_of(owner);
    //
    // }

    // fun store_student_entity(storage_ctx: &mut StorageContext, obj: Object<Student>) {
    //     let stu_store = storage_context::object_storage_mut(storage_ctx);
    //     object_storage::add(stu_store, obj)
    // }

    // fun add_transcript_item(
    //     storage_ctx: &mut StorageContext,
    //     grade: &mut Table<String, u64>,
    //     subject: String,
    //     score: u64
    // ) {
    //     table::add(grade, subject, score);
    //     event::emit(storage_ctx, TranscriptItemAddedEvent { item: Grade { subject, score } })
    // }

    // fun init(stroage_ctx: &mut StorageContext, owner: &signer) {
    // }

    // fun stu_example(ctx: &mut StorageContext, owner: &signer) {
    //     let name = string::utf8(b"Joe");
    //     let age = 20;
    //     let
    // }

    // entry fun create_student_entry(storage_cxt: &mut StorageContext, name: String, age: u32) {
    //     let stu = create_student(storage_cxt, name, age);
    //     store_student_entity(storage_cxt, stu);
    // }
}
