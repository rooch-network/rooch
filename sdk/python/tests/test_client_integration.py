#!/usr/bin/env python3
# Copyright (c) Rooch Network
# SPDX-License-Identifier: Apache-2.0

"""Integration tests for Rooch client"""

import asyncio
import base64
import pytest
from unittest.mock import AsyncMock, patch

from rooch.client.client import RoochClient
from rooch.transport import RoochTransport, RoochEnvironment
from rooch.crypto.keypair import KeyPair
from rooch.crypto.signer import Signer


class TestClientIntegration:
    """Integration tests for RoochClient class"""
    
    @pytest.mark.asyncio
    async def test_get_chain_id(self, mock_client):
        """Test getting chain ID"""
        # Chain ID is already mocked in conftest.py
        chain_id = await mock_client.get_chain_id()
        assert chain_id == 42
    
    @pytest.mark.asyncio
    async def test_get_account(self, mock_client, test_signer):
        """Test getting account information"""
        # Mock response is already set in conftest.py
        address = test_signer.get_address()
        account = await mock_client.account.get_account(address)
        
        assert account is not None
        assert "sequence_number" in account
        assert account["sequence_number"] == "0"
        assert "authentication_key" in account
    
    @pytest.mark.asyncio
    async def test_get_balances(self, mock_client, test_signer):
        """Test getting account balances"""
        # Mock response is already set in conftest.py
        address = test_signer.get_address()
        balances = await mock_client.account.get_balances(address)
        
        assert "balances" in balances
        assert len(balances["balances"]) > 0
        assert balances["balances"][0]["coin_type"] == "0x1::coin::ROOCH"
        assert balances["balances"][0]["amount"] == "1000000000"
    
    @pytest.mark.asyncio
    async def test_execute_view_function(self, mock_client):
        """Test executing a view function"""
        # Test with a simple function call
        result = await mock_client.transaction.execute_view_function(
            function_id="0x1::coin::balance",
            type_args=["0x1::coin::ROOCH"],
            args=[["0x123"]]
        )
        
        # Mock response is set in conftest.py to return "1000000000"
        assert result == "1000000000"
    
    @pytest.mark.asyncio
    async def test_execute_move_call(self, mock_client, test_signer):
        """Test executing a Move function call"""
        # Test executing a Move function
        result = await mock_client.execute_move_call(
            signer=test_signer,
            function_id="0x1::coin::transfer",
            type_args=["0x1::coin::ROOCH"],
            args=[["0x123", "100"]],
            max_gas_amount=10_000_000
        )
        
        # Check transaction hash was returned from mock
        assert isinstance(result, str)
        assert result.startswith("0x")
    
    @pytest.mark.asyncio
    async def test_get_transaction_by_hash(self, mock_client):
        """Test getting a transaction by hash"""
        # Mock response is already set in conftest.py
        tx_hash = "0x0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef"
        tx_info = await mock_client.transaction.get_transaction_by_hash(tx_hash)
        
        assert tx_info is not None
        assert "transaction" in tx_info
        assert "status" in tx_info
        assert tx_info["transaction"]["hash"] == tx_hash
        assert tx_info["status"]["status"] == "executed"
    
    @pytest.mark.asyncio
    async def test_wait_for_transaction(self, mock_client):
        """Test waiting for a transaction to complete"""
        # Mock get_transaction_by_hash to simulate waiting
        mock_responses = [
            # First call - transaction not found
            None,
            # Second call - transaction pending
            {
                "transaction": {"hash": "0x123"},
                "status": {"status": "pending"}
            },
            # Third call - transaction executed
            {
                "transaction": {"hash": "0x123"},
                "status": {"status": "executed"}
            }
        ]
        
        # Set up mock to return different responses in sequence
        mock_client.transaction.get_transaction_by_hash = AsyncMock(side_effect=mock_responses)
        
        # Wait for transaction
        result = await mock_client.transaction.wait_for_transaction("0x123", timeout_secs=5)
        
        # Verify result
        assert result["status"]["status"] == "executed"
        assert mock_client.transaction.get_transaction_by_hash.call_count == 3
    
    @pytest.mark.asyncio
    async def test_publish_module(self, mock_client, test_signer):
        """Test publishing a Move module"""
        # Mock module bytes
        module_bytes = b"mock_module_bytes"
        
        # Test publishing module
        result = await mock_client.publish_module(
            signer=test_signer,
            module_bytes=module_bytes,
            max_gas_amount=10_000_000
        )
        
        # Check transaction hash was returned from mock
        assert isinstance(result, str)
        assert result.startswith("0x")
    
    @pytest.mark.asyncio
    async def test_client_context_manager(self):
        """Test client as context manager"""
        # Create a mock transport
        mock_transport = AsyncMock()
        mock_transport.close = AsyncMock()
        
        # Create client with mock transport
        client = RoochClient(transport=mock_transport)
        
        # Use client as context manager
        async with client:
            # Do something with the client
            pass
        
        # Verify transport was closed
        mock_transport.close.assert_called_once()
    
    @pytest.mark.asyncio
    async def test_environment_urls(self):
        """Test environment URL resolution"""
        # Test that each environment resolves to an expected URL format
        with patch("rooch.client.http_transport.RoochHttpTransport") as mock_transport:
            # Local environment
            client = RoochClient(RoochEnvironment.LOCAL)
            assert "localhost" in mock_transport.call_args[0][0].lower()
            
            # Testnet environment
            client = RoochClient(RoochEnvironment.TESTNET)
            assert "testnet" in mock_transport.call_args[0][0].lower()
            
            # Mainnet environment
            client = RoochClient(RoochEnvironment.MAINNET)
            assert "rooch.network" in mock_transport.call_args[0][0].lower()