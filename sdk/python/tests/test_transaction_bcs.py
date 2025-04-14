#!/usr/bin/env python3
# Copyright (c) Rooch Network
# SPDX-License-Identifier: Apache-2.0

"""Tests for Transaction BCS serialization and deserialization"""

import pytest
from rooch.bcs.serializer import BcsSerializer, BcsDeserializer, Serializable, Deserializable
from rooch.transactions.types import (
    TypeTag, TypeTagCode, StructTag,
    ModuleId, FunctionId,
    MoveAction, MoveActionArgument, FunctionArgument,
    TransactionType, TransactionData,
    AuthenticatorType, TransactionAuthenticator, SignedTransaction,
    TransactionArgument
)
from rooch.address.rooch import RoochAddress
from typing import Any

def bcs_serialize(value: Serializable) -> bytes:
    """Helper function to serialize a value using BCS"""
    serializer = BcsSerializer()
    serializer.struct(value)
    return serializer.output()

def bcs_deserialize(cls: type, data: bytes) -> Any:
    """Helper function to deserialize a value using BCS"""
    deserializer = BcsDeserializer(data)
    return cls.deserialize(deserializer)

class TestTypeTagSerialization:
    """Tests for TypeTag serialization and deserialization"""
    
    def test_simple_type_tags(self):
        """Test serialization/deserialization of simple TypeTags"""
        type_tags = [
            TypeTag.bool(),
            TypeTag.u8(),
            TypeTag.u16(),
            TypeTag.u32(),
            TypeTag.u64(),
            TypeTag.u128(),
            TypeTag.u256(),
            TypeTag.address(),
        ]
        
        # Test each type tag
        for tag in type_tags:
            # Serialize
            serialized = bcs_serialize(tag)
            
            # Deserialize
            deserialized = bcs_deserialize(TypeTag, serialized)
            
            # Verify they match
            assert deserialized.type_code == tag.type_code
            assert deserialized.value == tag.value
    
    def test_vector_type_tag(self):
        """Test serialization/deserialization of vector TypeTag"""
        # Vector of u8
        vec_u8 = TypeTag.vector(TypeTag.u8())
        # Vector of vectors of u8
        vec_vec_u8 = TypeTag.vector(vec_u8)
        
        for tag in [vec_u8, vec_vec_u8]:
            # Serialize
            serialized = bcs_serialize(tag)
            
            # Deserialize
            deserialized = bcs_deserialize(TypeTag, serialized)
            
            # Verify nested structure is preserved
            assert deserialized.type_code == tag.type_code
            assert deserialized.value.type_code == tag.value.type_code
            if tag == vec_vec_u8:
                assert deserialized.value.value.type_code == tag.value.value.type_code
    
    def test_struct_type_tag(self):
        """Test serialization/deserialization of struct TypeTag"""
        # Create test data
        address = RoochAddress.from_hex_literal("0x1")
        struct_tag = StructTag(
            address=address,
            module="test_module",
            name="TestStruct",
            type_params=[]
        )
        type_tag = TypeTag.struct(struct_tag)
        
        # Test serialization/deserialization
        serialized = bcs_serialize(type_tag)
        deserialized = bcs_deserialize(TypeTag, serialized)
        
        # Verify the result
        assert deserialized.value.address.to_hex_literal() == "0x1"
        assert deserialized.value.module == "test_module"
        assert deserialized.value.name == "TestStruct"
        assert len(deserialized.value.type_params) == 0

    def test_complex_struct_type_tag(self):
        """Test serialization/deserialization of complex struct TypeTag"""
        # Create test data with nested type parameters
        address = RoochAddress.from_hex_literal("0x1")
        inner_struct = StructTag(
            address=address,
            module="inner_module",
            name="InnerStruct",
            type_params=[]
        )
        outer_struct = StructTag(
            address=address,
            module="outer_module",
            name="OuterStruct",
            type_params=[TypeTag.struct(inner_struct)]
        )
        type_tag = TypeTag.struct(outer_struct)
        
        # Test serialization/deserialization
        serialized = bcs_serialize(type_tag)
        deserialized = bcs_deserialize(TypeTag, serialized)
        
        # Verify the result
        assert deserialized.value.address.to_hex_literal() == "0x1"
        assert deserialized.value.module == "outer_module"
        assert deserialized.value.name == "OuterStruct"
        assert len(deserialized.value.type_params) == 1
        
        inner = deserialized.value.type_params[0].value
        assert inner.address.to_hex_literal() == "0x1"
        assert inner.module == "inner_module"
        assert inner.name == "InnerStruct"


class TestModuleAndFunctionIdSerialization:
    """Tests for ModuleId and FunctionId serialization and deserialization"""
    
    def test_module_id(self):
        """Test serialization/deserialization of ModuleId"""
        # Create test data
        address = RoochAddress.from_hex_literal("0x1")
        module_id = ModuleId(
            address=address,
            name="test_module"
        )
        
        # Test serialization/deserialization
        serialized = bcs_serialize(module_id)
        deserialized = bcs_deserialize(ModuleId, serialized)
        
        # Verify the result
        assert deserialized.address.to_hex_literal() == "0x1"
        assert deserialized.name == "test_module"
    
    def test_function_id(self):
        """Test serialization/deserialization of FunctionId"""
        # Create test data
        address = RoochAddress.from_hex_literal("0x1")
        module_id = ModuleId(
            address=address,
            name="test_module"
        )
        function_id = FunctionId(
            module_id=module_id,
            function_name="test_function"
        )
        
        # Test serialization/deserialization
        serialized = bcs_serialize(function_id)
        deserialized = bcs_deserialize(FunctionId, serialized)
        
        # Verify the result
        assert deserialized.module_id.address.to_hex_literal() == "0x1"
        assert deserialized.module_id.name == "test_module"
        assert deserialized.function_name == "test_function"


class TestMoveActionArgumentSerialization:
    """Tests for MoveActionArgument serialization and deserialization"""
    
    def test_function_move_action(self):
        """Test serialization/deserialization of FUNCTION MoveActionArgument"""
        # Create a test module and function ID
        address = RoochAddress.from_hex_literal("0x1")
        module_id = ModuleId(address=address, name="account")
        function_id = FunctionId(module_id=module_id, function_name="create_account")
        
        # Create type arguments and function arguments
        type_args = [TypeTag.u64(), TypeTag.address()]
        args = [
            TransactionArgument(TypeTagCode.ADDRESS, "0x1234567890"),  # Address argument
            TransactionArgument(TypeTagCode.U64, 100)  # U64 argument as integer
        ]
        
        # Create function argument
        function_arg = FunctionArgument(
            function_id=function_id,
            ty_args=type_args,
            args=args
        )
        
        # Create MoveActionArgument
        move_action_arg = MoveActionArgument(
            action=MoveAction.FUNCTION,
            args=function_arg
        )
        
        # Serialize
        serialized = bcs_serialize(move_action_arg)
        
        # Deserialize
        deserialized = bcs_deserialize(MoveActionArgument, serialized)
        
        # Verify they match
        assert deserialized.action == move_action_arg.action
        assert deserialized.args.function_id.module_id.address.to_hex_literal() == address.to_hex_literal()
        assert deserialized.args.function_id.module_id.name == function_arg.function_id.module_id.name
        assert deserialized.args.function_id.function_name == function_arg.function_id.function_name
        assert len(deserialized.args.ty_args) == len(function_arg.ty_args)
        
        # Print debug info
        print("\nOriginal args:")
        for arg in function_arg.args:
            print(f"type_tag: {arg.type_tag}, value: {arg.value}")
        print("\nDeserialized args:")
        for arg in deserialized.args.args:
            print(f"type_tag: {arg.type_tag}, value: {arg.value}")
            
        # Compare args one by one
        for i, (orig_arg, deser_arg) in enumerate(zip(function_arg.args, deserialized.args.args)):
            assert orig_arg.type_tag == deser_arg.type_tag, f"Type tag mismatch at index {i}"
            assert orig_arg.value == deser_arg.value, f"Value mismatch at index {i}"

    def test_module_bundle_move_action(self):
        """Test serialization/deserialization of MODULE_BUNDLE MoveActionArgument"""
        # Create a test module bundle
        module_bytes = b"mock module bytecode"
        
        # Create MoveActionArgument for module bundle
        move_action_arg = MoveActionArgument(
            action=MoveAction.MODULE_BUNDLE,
            args=module_bytes
        )
        
        # Serialize
        serialized = bcs_serialize(move_action_arg)
        
        # Deserialize
        deserialized = bcs_deserialize(MoveActionArgument, serialized)
        
        # Verify they match
        assert deserialized.action == move_action_arg.action
        assert deserialized.args == move_action_arg.args


class TestTransactionDataSerialization:
    """Tests for TransactionData serialization and deserialization"""
    
    def test_transaction_data(self):
        """Test serialization/deserialization of TransactionData"""
        # Create test data
        sender = RoochAddress.from_hex_literal("0x1234567890")
        sequence_number = 42
        chain_id = 1  # Local chain ID
        max_gas_amount = 1000000
        
        # Create a test function call action
        address = RoochAddress.from_hex_literal("0x1")
        module_id = ModuleId(address=address, name="account")
        function_id = FunctionId(module_id=module_id, function_name="create_account")
        function_arg = FunctionArgument(
            function_id=function_id,
            ty_args=[TypeTag.u64()],
            args=["100"]
        )
        move_action = MoveActionArgument(
            action=MoveAction.FUNCTION,
            args=function_arg
        )
        
        # Create TransactionData
        tx_data = TransactionData(
            tx_type=TransactionType.MOVE_ACTION,
            tx_arg=move_action,
            sequence_number=sequence_number,
            chain_id=chain_id,
            max_gas_amount=max_gas_amount
        )
        
        # Serialize
        serialized = bcs_serialize(tx_data)
        
        # Deserialize
        deserialized = bcs_deserialize(TransactionData, serialized)
        
        # Verify they match
        assert deserialized.tx_type == tx_data.tx_type
        assert deserialized.tx_arg.action == tx_data.tx_arg.action
        assert deserialized.sequence_number == tx_data.sequence_number
        assert deserialized.chain_id == tx_data.chain_id
        assert deserialized.max_gas_amount == tx_data.max_gas_amount

    def test_transaction_data_with_module_bundle(self):
        """Test serialization/deserialization of TransactionData with module bundle"""
        # Create test data
        sender = RoochAddress.from_hex_literal("0x1234567890")
        sequence_number = 42
        chain_id = 1  # Local chain ID
        max_gas_amount = 1000000
        
        # Create a test module bundle action
        module_bytes = b"mock module bytecode"
        move_action = MoveActionArgument(
            action=MoveAction.MODULE_BUNDLE,
            args=module_bytes
        )
        
        # Create TransactionData
        tx_data = TransactionData(
            tx_type=TransactionType.MOVE_ACTION,
            tx_arg=move_action,
            sequence_number=sequence_number,
            chain_id=chain_id,
            max_gas_amount=max_gas_amount
        )
        
        # Serialize
        serialized = bcs_serialize(tx_data)
        
        # Deserialize
        deserialized = bcs_deserialize(TransactionData, serialized)
        
        # Verify they match
        assert deserialized.tx_type == tx_data.tx_type
        assert deserialized.tx_arg.action == tx_data.tx_arg.action
        assert deserialized.sequence_number == tx_data.sequence_number
        assert deserialized.chain_id == tx_data.chain_id
        assert deserialized.max_gas_amount == tx_data.max_gas_amount


class TestSignedTransactionSerialization:
    """Tests for SignedTransaction serialization and deserialization"""
    
    def test_signed_transaction(self):
        """Test serialization/deserialization of SignedTransaction"""
        # Create test transaction data
        sender = RoochAddress.from_hex_literal("0x1234567890")
        sequence_number = 42
        chain_id = 1  # Local chain ID
        max_gas_amount = 1000000
        
        # Create a test function call action
        address = RoochAddress.from_hex_literal("0x1")
        module_id = ModuleId(address=address, name="account")
        function_id = FunctionId(module_id=module_id, function_name="create_account")
        function_arg = FunctionArgument(
            function_id=function_id,
            ty_args=[TypeTag.u64()],
            args=["100"]
        )
        move_action = MoveActionArgument(
            action=MoveAction.FUNCTION,
            args=function_arg
        )
        
        # Create TransactionData
        tx_data = TransactionData(
            tx_type=TransactionType.MOVE_ACTION,
            tx_arg=move_action,
            sequence_number=sequence_number,
            chain_id=chain_id,
            max_gas_amount=max_gas_amount
        )
        
        # Create mock authenticator
        auth_type = AuthenticatorType.ED25519
        public_key = b"mock public key"
        signature = b"mock signature"
        authenticator = TransactionAuthenticator(
            account_addr="0x1234567890",
            public_key=public_key,
            signature=signature,
            auth_type=auth_type
        )
        
        # Create SignedTransaction
        signed_tx = SignedTransaction(
            tx_data=tx_data,
            authenticator=authenticator
        )
        
        # Serialize
        serialized = bcs_serialize(signed_tx)
        
        # Deserialize
        deserialized = bcs_deserialize(SignedTransaction, serialized)
        
        # Verify transaction data
        assert deserialized.tx_data.tx_type == tx_data.tx_type
        assert deserialized.tx_data.tx_arg.action == tx_data.tx_arg.action
        assert deserialized.tx_data.sequence_number == tx_data.sequence_number
        assert deserialized.tx_data.chain_id == tx_data.chain_id
        assert deserialized.tx_data.max_gas_amount == tx_data.max_gas_amount
        
        # Verify authenticator
        assert deserialized.authenticator.auth_key.auth_type == authenticator.auth_key.auth_type
        assert deserialized.authenticator.auth_key.public_key == authenticator.auth_key.public_key
        assert deserialized.authenticator.signature == authenticator.signature

    def test_signed_transaction_with_module_bundle(self):
        """Test serialization/deserialization of SignedTransaction with module bundle"""
        # Create test transaction data
        sender = RoochAddress.from_hex_literal("0x1234567890")
        sequence_number = 42
        chain_id = 1  # Local chain ID
        max_gas_amount = 1000000
        
        # Create a test module bundle action
        module_bytes = b"mock module bytecode"
        move_action = MoveActionArgument(
            action=MoveAction.MODULE_BUNDLE,
            args=module_bytes
        )
        
        # Create TransactionData
        tx_data = TransactionData(
            tx_type=TransactionType.MOVE_ACTION,
            tx_arg=move_action,
            sequence_number=sequence_number,
            chain_id=chain_id,
            max_gas_amount=max_gas_amount
        )
        
        # Create mock authenticator
        auth_type = AuthenticatorType.ED25519
        public_key = b"mock public key"
        signature = b"mock signature"
        authenticator = TransactionAuthenticator(
            account_addr="0x1234567890",
            public_key=public_key,
            signature=signature,
            auth_type=auth_type
        )
        
        # Create SignedTransaction
        signed_tx = SignedTransaction(
            tx_data=tx_data,
            authenticator=authenticator
        )
        
        # Serialize
        serialized = bcs_serialize(signed_tx)
        
        # Deserialize
        deserialized = bcs_deserialize(SignedTransaction, serialized)
        
        # Verify transaction data
        assert deserialized.tx_data.tx_type == tx_data.tx_type
        assert deserialized.tx_data.tx_arg.action == tx_data.tx_arg.action
        assert deserialized.tx_data.sequence_number == tx_data.sequence_number
        assert deserialized.tx_data.chain_id == tx_data.chain_id
        assert deserialized.tx_data.max_gas_amount == tx_data.max_gas_amount
        
        # Verify authenticator
        assert deserialized.authenticator.auth_key.auth_type == authenticator.auth_key.auth_type
        assert deserialized.authenticator.auth_key.public_key == authenticator.auth_key.public_key
        assert deserialized.authenticator.signature == authenticator.signature 