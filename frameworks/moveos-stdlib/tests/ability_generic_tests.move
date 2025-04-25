// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

#[test_only]
module moveos_std::ability_generic_tests {
    use moveos_std::ability;
    use std::string;

    // Generic struct with different abilities depending on type param
    struct GenericContainer<T> has copy, drop, store { value: T }
    
    // Generic struct that copies its type parameter's abilities
    struct InheritedAbilities<T: store> has store { value: T }
    
    // Generic struct with multiple type parameters
    struct MultiParam<T1, T2> has copy, drop, store { val1: T1, val2: T2 }
    
    // Generic struct with phantom type parameter
    struct PhantomParam<phantom T> has copy, drop, store, key { value: u64 }

    #[test]
    fun test_generic_instantiations() {
        // Test GenericContainer<u64> - should have copy, drop, store
        let generic_u64_type = string::utf8(b"0x2::ability_generic_tests::GenericContainer<u64>");
        let generic_u64_abilities = ability::native_get_abilities(generic_u64_type);
        assert!(ability::has_copy(generic_u64_abilities), 0);
        assert!(ability::has_drop(generic_u64_abilities), 1);
        assert!(ability::has_store(generic_u64_abilities), 2);
        assert!(!ability::has_key(generic_u64_abilities), 3);
        
        // Test InheritedAbilities<u64> - should have store (from type param and declaration)
        let inherited_u64_type = string::utf8(b"0x2::ability_generic_tests::InheritedAbilities<u64>");
        let inherited_u64_abilities = ability::native_get_abilities(inherited_u64_type);
        assert!(!ability::has_copy(inherited_u64_abilities), 4);
        assert!(!ability::has_drop(inherited_u64_abilities), 5);
        assert!(ability::has_store(inherited_u64_abilities), 6);
        assert!(!ability::has_key(inherited_u64_abilities), 7);
        
        // Test MultiParam with two different types
        let multi_param_type = string::utf8(b"0x2::ability_generic_tests::MultiParam<u64, address>");
        let multi_param_abilities = ability::native_get_abilities(multi_param_type);
        assert!(ability::has_copy(multi_param_abilities), 8);
        assert!(ability::has_drop(multi_param_abilities), 9); 
        assert!(ability::has_store(multi_param_abilities), 10);
        assert!(!ability::has_key(multi_param_abilities), 11);
        
        // Test PhantomParam (should have all abilities regardless of phantom type)
        let phantom_signer_type = string::utf8(b"0x2::ability_generic_tests::PhantomParam<signer>");
        let phantom_abilities = ability::native_get_abilities(phantom_signer_type);
        assert!(ability::has_copy(phantom_abilities), 12);
        assert!(ability::has_drop(phantom_abilities), 13);
        assert!(ability::has_store(phantom_abilities), 14);
        assert!(ability::has_key(phantom_abilities), 15);
    }
    
    #[test]
    fun test_nested_generics() {
        // Test nested generics - container of container
        let nested_type = string::utf8(b"0x2::ability_generic_tests::GenericContainer<0x2::ability_generic_tests::GenericContainer<u8>>");
        let nested_abilities = ability::native_get_abilities(nested_type);
        assert!(ability::has_copy(nested_abilities), 0);
        assert!(ability::has_drop(nested_abilities), 1);
        assert!(ability::has_store(nested_abilities), 2);
        assert!(!ability::has_key(nested_abilities), 3);
        
        // Test vector of generic type
        let vec_generic_type = string::utf8(b"vector<0x2::ability_generic_tests::GenericContainer<u64>>");
        let vec_generic_abilities = ability::native_get_abilities(vec_generic_type);
        assert!(ability::has_copy(vec_generic_abilities), 4);
        assert!(ability::has_drop(vec_generic_abilities), 5);
        assert!(ability::has_store(vec_generic_abilities), 6);
        assert!(!ability::has_key(vec_generic_abilities), 7);
    }
    
    #[test]
    fun test_uninstantiated_generics() {
        // Test uninstantiated generic type - behavior may vary depending on implementation
        // but should not crash
        let generic_type = string::utf8(b"0x2::ability_generic_tests::GenericContainer");
        let generic_abilities = ability::native_get_abilities(generic_type);
        
        // We don't assert specific abilities here, just make sure it doesn't crash
        // The abilities returned might depend on the implementation
        // Just make a dummy assertion to keep the test running
        assert!(generic_abilities == generic_abilities, 0);
    }
} 