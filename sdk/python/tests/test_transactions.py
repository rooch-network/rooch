#!/usr/bin/env python3
# Copyright (c) Rooch Network
# SPDX-License-Identifier: Apache-2.0

"""Tests for transaction types module"""

import pytest
from rooch.transactions.types import (
    TypeTag, TypeTagCode, StructTag,
    ModuleId, FunctionId,
    MoveAction, MoveActionArgument, FunctionArgument,
    TransactionType, TransactionData,
    AuthenticatorType, AuthenticationKey, TransactionAuthenticator,
    SignedTransaction, TransactionArgument
)
from rooch.utils.hex import to_hex, from_hex

class TestTransactionArgument:
    """Tests for TransactionArgument class"""
    
    def test_create_transaction_argument(self):
        """Test creating transaction arguments"""
        # String argument
        str_arg = TransactionArgument(type_tag=0, value="test_string")
        assert str_arg.type_tag == 0
        assert str_arg.value == "test_string"
        
        # Number argument
        num_arg = TransactionArgument(type_tag=2, value=123)
        assert num_arg.type_tag == 2
        assert num_arg.value == 123
        
        # Boolean argument
        bool_arg = TransactionArgument(type_tag=9, value=True)
        assert bool_arg.type_tag == 9
        assert bool_arg.value is True
    
    def test_transaction_argument_to_dict(self):
        """Test converting transaction argument to dictionary"""
        arg = TransactionArgument(type_tag=0, value="test_value")
        arg_dict = arg.to_dict()
        
        assert isinstance(arg_dict, dict)
        assert arg_dict["type_tag"] == 0
        assert arg_dict["value"] == "test_value"
    
    def test_transaction_argument_from_dict(self):
        """Test creating transaction argument from dictionary"""
        arg_dict = {"type_tag": 2, "value": 123}
        arg = TransactionArgument.from_dict(arg_dict)
        
        assert arg.type_tag == 2
        assert arg.value == 123
        
        # Test with missing values
        arg = TransactionArgument.from_dict({})
        assert arg.type_tag == 0
        assert arg.value is None

class TestFunctionArgument:
    """Tests for FunctionArgument class"""
    
    def test_create_function_argument(self):
        """Test creating function arguments"""
        # Create transaction arguments
        args = [
            TransactionArgument(type_tag=0, value="0x123"),  # Address
            TransactionArgument(type_tag=2, value=100)       # Amount
        ]
        
        # Create function argument
        func_arg = FunctionArgument(
            function_id="0x1::coin::transfer",
            ty_args=["0x3::gas_coin::RGas"],
            args=args
        )
        
        assert str(func_arg.function_id) == "0x1::coin::transfer"
        assert len(func_arg.ty_args) == 1
        assert func_arg.ty_args[0] == "0x3::gas_coin::RGas"
        assert len(func_arg.args) == 2
        assert func_arg.args[0].value == "0x123"
        assert func_arg.args[1].value == 100
    
    def test_function_argument_to_dict(self):
        """Test converting function argument to dictionary"""
        # Create function argument
        func_arg = FunctionArgument(
            function_id="0x1::coin::transfer",
            ty_args=["0x3::gas_coin::RGas"],
            args=[
                TransactionArgument(type_tag=0, value="0x123"),
                TransactionArgument(type_tag=2, value=100)
            ]
        )
        
        # Convert to dictionary
        func_dict = func_arg.to_dict()
        
        assert isinstance(func_dict, dict)
        assert "0x1::coin::transfer" in func_dict["function_id"]
        assert len(func_dict["ty_args"]) == 1
        assert func_dict["ty_args"][0] == "0x3::gas_coin::RGas"
        assert len(func_dict["args"]) == 2
        assert func_dict["args"][0]["value"] == "0x123"
        assert func_dict["args"][1]["value"] == 100
    
    def test_function_argument_from_dict(self):
        """Test creating function argument from dictionary"""
        func_dict = {
            "function_id": "0x1::coin::transfer",
            "ty_args": ["0x3::gas_coin::RGas"],
            "args": [
                {"type_tag": 0, "value": "0x123"},
                {"type_tag": 2, "value": 100}
            ]
        }
        
        func_arg = FunctionArgument.from_dict(func_dict)
        
        assert str(func_arg.function_id) == "0x1::coin::transfer"
        assert len(func_arg.ty_args) == 1
        assert func_arg.ty_args[0] == "0x3::gas_coin::RGas"
        assert len(func_arg.args) == 2
        assert func_arg.args[0].type_tag == 0
        assert func_arg.args[0].value == "0x123"
        assert func_arg.args[1].type_tag == 2
        assert func_arg.args[1].value == 100
        
        # Test with empty dictionary
        func_arg = FunctionArgument.from_dict({})
        assert str(func_arg.function_id) == "0x1::empty::empty"
        assert len(func_arg.ty_args) == 0
        assert len(func_arg.args) == 0

class TestMoveActionArgument:
    """Tests for MoveActionArgument class"""
    
    def test_create_function_move_action(self):
        """Test creating move action for function call"""
        # Create function argument
        func_arg = FunctionArgument(
            function_id="0x1::coin::transfer",
            ty_args=["0x3::gas_coin::RGas"],
            args=[
                TransactionArgument(type_tag=0, value="0x123"),
                TransactionArgument(type_tag=2, value=100)
            ]
        )
        
        # Create move action
        move_action = MoveActionArgument(
            action=MoveAction.FUNCTION,
            args=func_arg
        )
        
        assert move_action.action == MoveAction.FUNCTION
        assert isinstance(move_action.args, FunctionArgument)
        assert str(move_action.args.function_id) == "0x1::coin::transfer"
    
    def test_create_script_move_action(self):
        """Test creating move action for script execution"""
        # Create script move action
        script_code = "script_bytecode_here"
        move_action = MoveActionArgument(
            action=MoveAction.SCRIPT,
            args=script_code
        )
        
        assert move_action.action == MoveAction.SCRIPT
        assert move_action.args == "script_bytecode_here"
    
    def test_move_action_argument_to_dict(self):
        """Test converting move action to dictionary"""
        # Function move action
        func_arg = FunctionArgument(
            function_id="0x1::coin::transfer",
            ty_args=[],
            args=[TransactionArgument(type_tag=0, value="0x123")]
        )
        
        func_action = MoveActionArgument(action=MoveAction.FUNCTION, args=func_arg)
        func_dict = func_action.to_dict()
        
        assert func_dict["action"] == MoveAction.FUNCTION
        assert "0x1::coin::transfer" in func_dict["args"]["function_id"]
        
        # Script move action
        script_action = MoveActionArgument(action=MoveAction.SCRIPT, args="script_code")
        script_dict = script_action.to_dict()
        
        assert script_dict["action"] == MoveAction.SCRIPT
        assert script_dict["args"] == "script_code"
    
    def test_move_action_argument_from_dict(self):
        """Test creating move action from dictionary"""
        # Function move action
        func_dict = {
            "action": MoveAction.FUNCTION,
            "args": {
                "function_id": "0x1::coin::transfer",
                "ty_args": [],
                "args": [{"type_tag": 0, "value": "0x123"}]
            }
        }
        
        func_action = MoveActionArgument.from_dict(func_dict)
        assert func_action.action == MoveAction.FUNCTION
        assert isinstance(func_action.args, FunctionArgument)
        assert str(func_action.args.function_id) == "0x1::coin::transfer"
        
        # Script move action
        script_dict = {
            "action": MoveAction.SCRIPT,
            "args": "script_code"
        }
        
        script_action = MoveActionArgument.from_dict(script_dict)
        assert script_action.action == MoveAction.SCRIPT
        assert script_action.args == b"script_code" or script_action.args == "script_code"

class TestTransactionData:
    """Tests for TransactionData class"""
    
    def test_create_move_action_transaction(self):
        """Test creating transaction data for move action"""
        # Create function argument
        func_arg = FunctionArgument(
            function_id="0x1::coin::transfer",
            ty_args=["0x3::gas_coin::RGas"],
            args=[
                TransactionArgument(type_tag=0, value="0x123"),
                TransactionArgument(type_tag=2, value=100)
            ]
        )
        
        # Create move action
        move_action = MoveActionArgument(
            action=MoveAction.FUNCTION,
            args=func_arg
        )
        
        # Create transaction data
        tx_data = TransactionData(
            tx_type=TransactionType.MOVE_ACTION,
            tx_arg=move_action,
            sequence_number=10,
            max_gas_amount=1000000,
            gas_unit_price=1,
            expiration_timestamp_secs=1650000000,
            chain_id=42
        )
        
        assert tx_data.tx_type == TransactionType.MOVE_ACTION
        assert isinstance(tx_data.tx_arg, MoveActionArgument)
        assert tx_data.sequence_number == 10
        assert tx_data.max_gas_amount == 1000000
        assert tx_data.gas_unit_price == 1
        assert tx_data.expiration_timestamp_secs == 1650000000
        assert tx_data.chain_id == 42
    
    def test_create_module_transaction(self):
        """Test creating transaction data for module publishing"""
        # Create module bytes
        module_bytes = b"module_bytecode"
        
        # Create transaction data
        tx_data = TransactionData(
            tx_type=TransactionType.MOVE_MODULE_TRANSACTION,
            tx_arg=module_bytes,
            sequence_number=10,
            max_gas_amount=1000000,
            gas_unit_price=1,
            expiration_timestamp_secs=1650000000,
            chain_id=42
        )
        
        assert tx_data.tx_type == TransactionType.MOVE_MODULE_TRANSACTION
        assert tx_data.tx_arg == b"module_bytecode"
        assert tx_data.sequence_number == 10
    
    def test_transaction_data_to_dict(self):
        """Test converting transaction data to dictionary"""
        # Move action transaction
        func_arg = FunctionArgument(
            function_id="0x1::coin::transfer",
            ty_args=[],
            args=[TransactionArgument(type_tag=0, value="0x123")]
        )
        move_action = MoveActionArgument(action=MoveAction.FUNCTION, args=func_arg)
        
        tx_data = TransactionData(
            tx_type=TransactionType.MOVE_ACTION,
            tx_arg=move_action,
            sequence_number=10,
            max_gas_amount=1000000,
            chain_id=42
        )
        
        tx_dict = tx_data.to_dict()
        assert tx_dict["tx_type"] == TransactionType.MOVE_ACTION
        assert tx_dict["sequence_number"] == "10"
        assert tx_dict["max_gas_amount"] == "1000000"
        assert tx_dict["chain_id"] == 42
        assert tx_dict["tx_arg"]["action"] == MoveAction.FUNCTION
        
        # Module transaction
        module_tx = TransactionData(
            tx_type=TransactionType.MOVE_MODULE_TRANSACTION,
            tx_arg=b"module_bytecode",
            sequence_number=10
        )
        
        module_dict = module_tx.to_dict()
        assert module_dict["tx_type"] == TransactionType.MOVE_MODULE_TRANSACTION
        assert module_dict["tx_arg"].startswith("0x")  # Hex encoded bytes
    
    def test_transaction_data_from_dict(self):
        """Test creating transaction data from dictionary"""
        # Move action transaction
        tx_dict = {
            "tx_type": TransactionType.MOVE_ACTION,
            "tx_arg": {
                "action": MoveAction.FUNCTION,
                "args": {
                    "function_id": "0x1::coin::transfer",
                    "ty_args": [],
                    "args": [{"type_tag": 0, "value": "0x123"}]
                }
            },
            "sequence_number": "10",
            "max_gas_amount": "1000000",
            "gas_unit_price": "1",
            "expiration_timestamp_secs": "1650000000",
            "chain_id": 42
        }
        
        tx_data = TransactionData.from_dict(tx_dict)
        assert tx_data.tx_type == TransactionType.MOVE_ACTION
        assert isinstance(tx_data.tx_arg, MoveActionArgument)
        assert tx_data.sequence_number == 10
        assert tx_data.max_gas_amount == 1000000
        assert tx_data.chain_id == 42
        
        # Module transaction with hex string
        module_dict = {
            "tx_type": TransactionType.MOVE_MODULE_TRANSACTION,
            "tx_arg": "0x6d6f64756c655f627974656f6465",  # Hex for "module_bytecode"
            "sequence_number": 10
        }
        
        module_tx = TransactionData.from_dict(module_dict)
        assert module_tx.tx_type == TransactionType.MOVE_MODULE_TRANSACTION
        assert isinstance(module_tx.tx_arg, bytes)

class TestAuthenticationKey:
    """Tests for AuthenticationKey class"""
    
    def test_create_authentication_key(self):
        """Test creating authentication key"""
        # From hex string
        auth_key = AuthenticationKey(
            auth_type=AuthenticatorType.ED25519,
            public_key="0xabcdef1234567890"
        )
        
        assert auth_key.auth_type == AuthenticatorType.ED25519
        assert isinstance(auth_key.public_key, bytes)
        assert auth_key.public_key == from_hex("0xabcdef1234567890")
        
        # From bytes
        key_bytes = b"\x12\x34\x56\x78\x90"
        auth_key2 = AuthenticationKey(
            auth_type=AuthenticatorType.SECP256K1,
            public_key=key_bytes
        )
        
        assert auth_key2.auth_type == AuthenticatorType.SECP256K1
        assert auth_key2.public_key == key_bytes
    
    def test_authentication_key_to_dict(self):
        """Test converting authentication key to dictionary"""
        key_bytes = b"\x12\x34\x56\x78\x90"
        auth_key = AuthenticationKey(
            auth_type=AuthenticatorType.ED25519,
            public_key=key_bytes
        )
        
        key_dict = auth_key.to_dict()
        assert key_dict["auth_type"] == AuthenticatorType.ED25519
        assert key_dict["public_key"] == to_hex(key_bytes)

class TestTransactionAuthenticator:
    """Tests for TransactionAuthenticator class"""
    
    def test_create_transaction_authenticator(self):
        """Test creating transaction authenticator"""
        # From hex strings
        auth = TransactionAuthenticator(
            account_addr="0x123456",
            public_key="0xabcdef12",
            signature="0x9876fedc",
            auth_type=AuthenticatorType.ED25519
        )
        
        assert auth.account_addr == "0x123456"
        assert auth.auth_key.auth_type == AuthenticatorType.ED25519
        assert isinstance(auth.auth_key.public_key, bytes)
        assert isinstance(auth.signature, bytes)
        
        # From bytes
        auth2 = TransactionAuthenticator(
            account_addr="0x123456",
            public_key=b"\x12\x34\x56",
            signature=b"\x98\x76\xfe",
            auth_type=AuthenticatorType.SECP256K1
        )
        
        assert auth2.account_addr == "0x123456"
        assert auth2.auth_key.auth_type == AuthenticatorType.SECP256K1
        assert auth2.auth_key.public_key == b"\x12\x34\x56"
        assert auth2.signature == b"\x98\x76\xfe"
    
    def test_transaction_authenticator_to_dict(self):
        """Test converting transaction authenticator to dictionary"""
        auth = TransactionAuthenticator(
            account_addr="0x123456",
            public_key="0xabcdef12",
            signature="0x9876fedc"
        )
        
        auth_dict = auth.to_dict()
        assert auth_dict["account_addr"] == "0x123456"
        assert auth_dict["auth_key"]["auth_type"] == AuthenticatorType.ED25519
        assert auth_dict["auth_key"]["public_key"] == to_hex(from_hex("0xabcdef12"))
        assert auth_dict["signature"] == to_hex(from_hex("0x9876fedc"))
    
    def test_transaction_authenticator_from_dict(self):
        """Test creating transaction authenticator from dictionary"""
        auth_dict = {
            "account_addr": "0x123456",
            "auth_key": {
                "auth_type": AuthenticatorType.ED25519,
                "public_key": "0xabcdef12"
            },
            "signature": "0x9876fedc"
        }
        
        auth = TransactionAuthenticator.from_dict(auth_dict)
        assert auth.account_addr == "0x123456"
        assert auth.auth_key.auth_type == AuthenticatorType.ED25519
        
        # Test with missing values
        auth = TransactionAuthenticator.from_dict({})
        assert auth.account_addr == ""
        assert auth.auth_key.auth_type == AuthenticatorType.ED25519

class TestSignedTransaction:
    """Tests for SignedTransaction class"""
    
    def test_create_signed_transaction(self):
        """Test creating signed transaction"""
        # Create transaction data
        func_arg = FunctionArgument(
            function_id="0x1::coin::transfer",
            ty_args=[],
            args=[TransactionArgument(type_tag=0, value="0x123")]
        )
        move_action = MoveActionArgument(action=MoveAction.FUNCTION, args=func_arg)
        
        tx_data = TransactionData(
            tx_type=TransactionType.MOVE_ACTION,
            tx_arg=move_action,
            sequence_number=10
        )
        
        # Create authenticator
        auth = TransactionAuthenticator(
            account_addr="0x123456",
            public_key="0xabcdef12",
            signature="0x9876fedc"
        )
        
        # Create signed transaction
        signed_tx = SignedTransaction(tx_data=tx_data, authenticator=auth)
        
        assert signed_tx.tx_data == tx_data
        assert signed_tx.authenticator == auth
    
    def test_signed_transaction_to_dict(self):
        """Test converting signed transaction to dictionary"""
        # Create signed transaction
        tx_data = TransactionData(
            tx_type=TransactionType.MOVE_MODULE_TRANSACTION,
            tx_arg=b"module_bytes",
            sequence_number=10
        )
        
        auth = TransactionAuthenticator(
            account_addr="0x123456",
            public_key="0xabcdef12",
            signature="0x9876fedc"
        )
        
        signed_tx = SignedTransaction(tx_data=tx_data, authenticator=auth)
        
        # Convert to dictionary
        tx_dict = signed_tx.to_dict()
        assert "tx_data" in tx_dict
        assert "authenticator" in tx_dict
        assert tx_dict["tx_data"]["tx_type"] == TransactionType.MOVE_MODULE_TRANSACTION
        assert tx_dict["authenticator"]["account_addr"] == "0x123456"
    
    def test_signed_transaction_from_dict(self):
        """Test creating signed transaction from dictionary"""
        tx_dict = {
            "tx_data": {
                "tx_type": TransactionType.MOVE_ACTION,
                "tx_arg": {
                    "action": MoveAction.FUNCTION,
                    "args": {
                        "function_id": "0x1::coin::transfer",
                        "ty_args": [],
                        "args": [{"type_tag": 0, "value": "0x123"}]
                    }
                },
                "sequence_number": "10"
            },
            "authenticator": {
                "account_addr": "0x123456",
                "auth_key": {
                    "auth_type": AuthenticatorType.ED25519,
                    "public_key": "0xabcdef12"
                },
                "signature": "0x9876fedc"
            }
        }
        
        signed_tx = SignedTransaction.from_dict(tx_dict)
        assert signed_tx.tx_data.tx_type == TransactionType.MOVE_ACTION
        assert signed_tx.tx_data.sequence_number == 10
        assert signed_tx.authenticator.account_addr == "0x123456"