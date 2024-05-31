//# init --addresses tester=0x42

//call function use move value style argument
//# run --signers tester --args 1u8 1u16 1u32 1u64 1u128 1u256 @0x42 b"hello"
script {
    
    fun main(
        _v_u8: u8, 
        _v_u16: u16,
        _v_u32: u32,
        _v_u64: u64,
        _v_u128: u128,
        _v_u256: u256,
        _v_address: address,
        _v_string: std::string::String,
        ) {
    }
}

//call function use FunctionArgs style argument
//#run --signers tester --args u8:1 u16:1 u32:1 u64:1 u128:1 u256:1 address:0x42 string:hello object_id:0x5921974509dbe44ab84328a625f4a6580a5f89dff3e4e2dec448cb2b1c7f5b9 object:moveos_std::object::Timestamp
script {
    
    fun main(
        _v_u8: u8, 
        _v_u16: u16,
        _v_u32: u32,
        _v_u64: u64,
        _v_u128: u128,
        _v_u256: u256,
        _v_address: address,
        _v_string: std::string::String,
        _v_object_id: moveos_std::object::ObjectID,
        _v_object: &moveos_std::object::Object<moveos_std::object::Timestamp>,
        ) {
    }
}

//TODO FIXME currently, can not support vector as tests args
//call function use FunctionArgs style argument vector
////# run --signers tester --args vector<u8>:1,2,3 vector<u16>:1,2,3 vector<u32>:1,2,3 vector<u64>:1,2,3 vector<u128>:1,2,3 vector<256>:1,2,3 vector<address>:0x42,0x43,0x44 vector<string>:hello,world vector<object_id>:0x1,0x2,0x3
// script {
    
//     fun main(
//         _v_u8: vector<u8>, 
//         _v_u16: vector<u16>,
//         _v_u32: vector<u32>,
//         _v_u64: vector<u64>,
//         _v_u128: vector<u128>,
//         _v_u256: vector<u256>,
//         _v_address: vector<address>,
//         _v_string: vector<std::string::String>,
//         _v_object_id: vector<moveos_std::object::ObjectID>,
//         ) {
//     }
// }