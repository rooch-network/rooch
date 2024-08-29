//# init --addresses test=0x42

//check the module store object id
//# run --signers test
script {
    fun main() {
        let object_id = moveos_std::object::named_object_id<moveos_std::module_store::ModuleStore>();
        std::debug::print(&object_id);
    }
}