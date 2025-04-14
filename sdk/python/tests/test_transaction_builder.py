#!/usr/bin/env python3
# Copyright (c) Rooch Network
# SPDX-License-Identifier: Apache-2.0

"""Tests for transaction builder module"""

import pytest
import time # Import time for expiration test
from unittest.mock import MagicMock, AsyncMock, patch

from rooch.transactions.builder import TransactionBuilder
from rooch.transactions.types import (
    TypeTag, TypeTagCode, StructTag,
    ModuleId, FunctionId,
    MoveAction, MoveActionArgument, FunctionArgument,
    TransactionType, TransactionData
)
from rooch.bcs.serializer import Args # Import Args helper
from rooch.crypto.keypair import KeyPair
from rooch.crypto.signer import Signer, RoochSigner
from rooch.bcs.serializer import BcsSerializer


class TestTransactionBuilder:
    """Tests for TransactionBuilder class"""
    
    def setup_method(self):
        """Setup before each test"""
        # Create test keypair and signer
        self.keypair = KeyPair.generate()
        self.signer = RoochSigner(self.keypair)
        self.address = self.signer.get_address()
        self.sequence_number = 10 # Example sequence number for tests
        self.chain_id = 42 # Example chain ID for tests
        self.max_gas = 1_000_000
        self.gas_price = 1

        # Create transaction builder with required args
        self.builder = TransactionBuilder(
            sender_address=self.address,
            sequence_number=self.sequence_number,
            chain_id=self.chain_id,
            max_gas_amount=self.max_gas,
            gas_unit_price=self.gas_price
            # expiration_timestamp_secs can use defaults or be set per test
        )
        # Verify builder attributes are set correctly
        assert self.builder.sender_address == self.address
        assert self.builder.sequence_number == self.sequence_number
        assert self.builder.chain_id == self.chain_id
        assert self.builder.max_gas_amount == self.max_gas
        assert self.builder.gas_unit_price == self.gas_price

    def test_build_move_call_transaction(self):
        """Test building a Move function call transaction"""
        # Define function call parameters
        function_id = "0x1::coin::transfer"
        ty_args = ["0x3::gas_coin::RGas"]
        # Use Args helper for arguments
        call_args = [Args.address("0x123"), Args.u64(100)]

        # 1. Build payload
        payload = self.builder.build_function_payload(
            function_id=function_id,
            ty_args=ty_args,
            args=call_args # Pass the list returned by Args helpers
        )

        # 2. Build transaction data
        tx_data = self.builder.build_move_action_tx(payload)

        # Verify transaction data type and payload
        assert isinstance(tx_data, TransactionData)
        assert tx_data.tx_type == TransactionType.MOVE_ACTION
        assert isinstance(tx_data.tx_arg, MoveActionArgument)

        # Verify payload details within TransactionData
        action_arg = tx_data.tx_arg
        assert action_arg.action == MoveAction.FUNCTION
        function_arg = action_arg.args
        assert isinstance(function_arg, FunctionArgument)
        assert function_arg.function_id == function_id
        # Compare each type argument individually
        assert len(function_arg.ty_args) == len(ty_args)
        for actual, expected in zip(function_arg.ty_args, ty_args):
            assert actual == expected

    def test_build_module_publish_transaction(self):
        """Test building a module publish transaction"""
        # Mock module bytes
        module_bytes = b"mock_module_bytes"

        # Build transaction data directly
        tx_data = self.builder.build_module_publish_tx(module_bytes)

        # Verify transaction data type and payload
        assert isinstance(tx_data, TransactionData)
        assert tx_data.tx_type == TransactionType.MOVE_MODULE_TRANSACTION
        assert tx_data.tx_arg == module_bytes

    def test_build_raw_transaction(self):
        """Test building a raw transaction using build_function_payload"""
        # Define function call parameters using Args
        function_id = "0x1::test::custom_function"
        ty_args = ["0x1::test::TestType"]
        call_args = [Args.address("0xabc"), Args.u64(123)]

        # 1. Build payload using the standard helper
        move_action_arg = self.builder.build_function_payload(
            function_id=function_id,
            ty_args=ty_args,
            args=call_args
        )

        # 2. Build transaction data
        tx_data = self.builder.build_move_action_tx(move_action_arg)

        # Verify transaction data type and payload
        assert isinstance(tx_data, TransactionData)
        assert tx_data.tx_type == TransactionType.MOVE_ACTION

        # Verify payload details
        action_payload = tx_data.tx_arg
        assert isinstance(action_payload, MoveActionArgument)
        assert action_payload.action == MoveAction.FUNCTION
        func_payload = action_payload.args
        assert isinstance(func_payload, FunctionArgument)
        assert func_payload.function_id == function_id
        # Compare each type argument individually
        assert len(func_payload.ty_args) == len(ty_args)
        for actual, expected in zip(func_payload.ty_args, ty_args):
            assert actual == expected

    def test_chain_id_caching(self):
        """Test that chain ID is set correctly during init"""
        # Builder is initialized with chain_id in setup_method
        assert self.builder.chain_id == self.chain_id

        # Build a simple payload and tx_data
        payload = self.builder.build_function_payload("0x1::m::f")
        tx_data = self.builder.build_move_action_tx(payload)

        # Verify the chain_id used in the transaction data matches the builder's
        # This check is implicit now as tx_data gets it from builder
        assert hasattr(tx_data, 'chain_id')
        assert tx_data.chain_id == self.builder.chain_id

    def test_expiration_timestamp_calculation(self):
        """Test expiration timestamp calculation (now done via builder init or static method)"""
        expiration_delta = 3600 # 1 hour
        current_time = 1000000000

        # Option 1: Set expiration during builder init
        with patch('time.time', return_value=current_time):
            expected_expiry = int(time.time()) + expiration_delta
            builder_with_expiry = TransactionBuilder(
                sender_address=self.address,
                sequence_number=self.sequence_number,
                chain_id=self.chain_id,
                max_gas_amount=self.max_gas,
                gas_unit_price=self.gas_price,
                # Calculate expiration based on delta
                expiration_timestamp_secs=expected_expiry
            )

        payload = builder_with_expiry.build_function_payload("0x1::test::function")
        tx_data = builder_with_expiry.build_move_action_tx(payload)

        # Verify expiration timestamp in TransactionData
        assert hasattr(tx_data, 'expiration_timestamp_secs')
        assert tx_data.expiration_timestamp_secs == expected_expiry

        # Option 2: Use static method with_default_account
        with patch('time.time', return_value=current_time):
            expected_expiry_static = int(time.time()) + expiration_delta
            builder_from_static = TransactionBuilder.with_default_account(
                signer=self.signer,
                sequence_number=self.sequence_number,
                chain_id=self.chain_id,
                max_gas_amount=self.max_gas,
                gas_unit_price=self.gas_price,
                expiration_delta_secs=expiration_delta
            )

        payload_static = builder_from_static.build_function_payload("0x1::test::function")
        tx_data_static = builder_from_static.build_move_action_tx(payload_static)

        # Verify expiration timestamp from static method builder
        assert hasattr(tx_data_static, 'expiration_timestamp_secs')
        assert tx_data_static.expiration_timestamp_secs == expected_expiry_static