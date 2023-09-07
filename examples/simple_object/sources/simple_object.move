module simple_object::simple_object {
    // use std::string;
    use std::string::String;
    use moveos_std::object_storage;
    use moveos_std::tx_context;
    use moveos_std::event;
    use moveos_std::storage_context::{Self, StorageContext};
    use moveos_std::object_id::ObjectID;
    use moveos_std::object::{Self, Object};
    use moveos_std::table::{Self, Table};

    struct Student has key {
        name: String,
        age: u32,
        transcript: Table<String, u64>
    }

    struct StudentCreatedEvent {
        obj_id: ObjectID,
        name: String,
        age: u32,
    }

    struct Grade<K, V> {
        subject: K,
        score: V
    }

    struct TranscriptItemAddedEvent {
        item: Grade<String, u64>
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

    fun create_student(
        storage_ctx: &mut StorageContext,
        name: String,
        age: u32,
    ): Object<Student> {
        let tx_ctx = storage_context::tx_context_mut(storage_ctx);
        let owner = tx_context::sender(tx_ctx);
        let stu_pros = Student { name, age, transcript: table::new(tx_ctx) };
        let student = object::new(tx_ctx, owner, stu_pros);
        student
    }

    fun store_student_entity(storage_ctx: &mut StorageContext, obj: Object<Student>) {
        let stu_store = storage_context::object_storage_mut(storage_ctx);
        object_storage::add(stu_store, obj)
    }

    fun add_transcript_item(
        storage_ctx: &mut StorageContext,
        grade: &mut Table<String, u64>,
        subject: String,
        score: u64
    ) {
        table::add(grade, subject, score);
        event::emit(storage_ctx, TranscriptItemAddedEvent { item: Grade { subject, score } })
    }

    // fun init(stroage_ctx: &mut StorageContext, owner: &signer) {
    // }

    // fun stu_example(ctx: &mut StorageContext, owner: &signer) {
    //     let name = string::utf8(b"Joe");
    //     let age = 20;
    //     let
    // }

    entry fun create_student_entry(storage_cxt: &mut StorageContext, name: String, age: u32) {
        let stu = create_student(storage_cxt, name, age);
        store_student_entity(storage_cxt, stu);
    }
}
