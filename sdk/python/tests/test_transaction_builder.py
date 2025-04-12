#!/usr/bin/env python3
# Copyright (c) Rooch Network
# SPDX-License-Identifier: Apache-2.0

"""Tests for transaction builder module"""

import pytest
from unittest.mock import MagicMock, AsyncMock, patch

from rooch.transactions.builder import TransactionBuilder
from rooch.transactions.types import TransactionData, MoveActionArgument, FunctionArgument, TransactionArgument, MoveAction, TransactionType
from rooch.crypto.keypair import KeyPair
from rooch.crypto.signer import Signer
from rooch.bcs.serializer import BcsSerializer


class TestTransactionBuilder:
    """Tests for TransactionBuilder class"""
    
    def setup_method(self):
        """Setup before each test"""
        # Create mock client for the transaction builder
        self.mock_client = MagicMock()
        self.mock_client.get_chain_id = AsyncMock(return_value=42)
        self.mock_client.account = MagicMock()
        self.mock_client.account.get_account = AsyncMock(return_value={
            "sequence_number": "10"
        })
        
        # Create test keypair and signer
        self.keypair = KeyPair.generate()
        self.signer = Signer(self.keypair)
        self.address = self.signer.get_address()
        
        # Create transaction builder
        self.builder = TransactionBuilder(self.mock_client)
    
    @pytest.mark.asyncio
    async def test_build_move_call_transaction(self):
        """Test building a Move function call transaction"""
        # Build transaction
        tx_data = await self.builder.build_move_call_transaction(
            self.signer,
            "0x1::coin::transfer",
            ["0x1::coin::ROOCH"],
            [["0x123", "100"]],
            1000000,  # max gas amount
            1,  # gas unit price
            3600  # expiration
        )
        
        # Verify transaction data
        assert tx_data["sender"] == self.address
        assert tx_data["sequence_number"] == 10
        assert tx_data["max_gas_amount"] == 1000000
        assert tx_data["gas_unit_price"] == 1
        
        # Verify payload
        payload = tx_data["payload"]
        assert isinstance(payload, dict)
        assert payload["type"] == "entry_function_payload"
        assert payload["function"] == "0x1::coin::transfer"
        assert payload["type_arguments"] == ["0x1::coin::ROOCH"]
        assert len(payload["arguments"]) == 2
        assert payload["arguments"][0] == "0x123"
        assert payload["arguments"][1] == "100"
    
    @pytest.mark.asyncio
    async def test_build_module_publish_transaction(self):
        """Test building a module publish transaction"""
        # Mock module bytes
        module_bytes = b"mock_module_bytes"
        
        # Build transaction
        tx_data = await self.builder.build_module_publish_transaction(
            self.signer,
            module_bytes,
            1000000,  # max gas amount
            1,  # gas unit price
            3600  # expiration
        )
        
        # Verify transaction data
        assert tx_data["sender"] == self.address
        assert tx_data["sequence_number"] == 10
        assert tx_data["max_gas_amount"] == 1000000
        assert tx_data["gas_unit_price"] == 1
        
        # Verify payload
        payload = tx_data["payload"]
        assert isinstance(payload, dict)
        assert payload["type"] == "module_payload"
        assert "modules" in payload
        assert isinstance(payload["modules"], list)
        assert len(payload["modules"]) == 1
        # Module bytes are base64 encoded
        assert isinstance(payload["modules"][0], str)
    
    @pytest.mark.asyncio
    async def test_build_raw_transaction(self):
        """Test building a raw transaction with custom payload"""
        # Create custom function argument
        function_arg = FunctionArgument(
            function_id="0x1::test::custom_function",
            ty_args=["0x1::test::TestType"],
            args=[
                TransactionArgument(type_tag=0, value="test1"),
                TransactionArgument(type_tag=2, value=123)
            ]
        )
        
        # Create move action argument
        move_action_arg = MoveActionArgument(
            action=MoveAction.FUNCTION,
            args=function_arg
        )
        
        # Build transaction
        tx_data = await self.builder.build_raw_transaction(
            self.signer,
            move_action_arg,
            1000000,  # max gas amount
            1,  # gas unit price
            3600  # expiration
        )
        
        # Verify transaction data
        assert tx_data["sender"] == self.address
        assert tx_data["sequence_number"] == 10
        assert tx_data["max_gas_amount"] == 1000000
        assert tx_data["gas_unit_price"] == 1
        
        # Test depends on actual implementation of build_raw_transaction
        # This may need adjustment based on how your builder actually works
    
    @pytest.mark.asyncio
    async def test_chain_id_caching(self):
        """Test that chain ID is cached correctly"""
        # First call should fetch chain ID
        await self.builder.build_move_call_transaction(
            self.signer,
            "0x1::test::function",
            [],
            [],
            1000000,
            1,
            3600
        )
        
        # Verify chain ID was fetched once
        self.mock_client.get_chain_id.assert_called_once()
        
        # Reset mock
        self.mock_client.get_chain_id.reset_mock()
        
        # Second call should use cached chain ID
        await self.builder.build_move_call_transaction(
            self.signer,
            "0x1::test::function",
            [],
            [],
            1000000,
            1,
            3600
        )
        
        # Verify chain ID was not fetched again
        self.mock_client.get_chain_id.assert_not_called()
    
    @pytest.mark.asyncio
    async def test_expiration_timestamp_calculation(self):
        """Test expiration timestamp calculation"""
        # Mock current timestamp
        with patch('time.time', return_value=1000000000):
            tx_data = await self.builder.build_move_call_transaction(
                self.signer,
                "0x1::test::function",
                [],
                [],
                1000000,
                1,
                3600  # 1 hour expiration
            )
            
            # Verify expiration timestamp
            assert tx_data["expiration_timestamp_secs"] == 1000000000 + 3600