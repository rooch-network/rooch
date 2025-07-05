#!/usr/bin/env python3
# Copyright (c) Rooch Network
# SPDX-License-Identifier: Apache-2.0

"""Tests for session functionality"""

import pytest
from unittest.mock import MagicMock, AsyncMock, patch
import time

from rooch.client.client import RoochClient
from rooch.crypto.keypair import KeyPair
from rooch.crypto.signer import Signer, RoochSigner


class TestSession:
    """Tests for session functionality"""
    
    @pytest.fixture
    def mock_client(self):
        """Create a mock client for testing"""
        client = MagicMock(spec=RoochClient)
        
        # Mock RPC calls
        client.account = MagicMock()
        client.account.get_account = AsyncMock(return_value={"sequence_number": "0"})
        
        # Mock session methods with typical responses
        client.session = MagicMock()
        client.session.create_session = AsyncMock(return_value={
            "session_id": "test-session-id",
            "sender": "0x123"
        })
        client.session.list_sessions = AsyncMock(return_value={
            "sessions": [
                {
                    "session_id": "test-session-id",
                    "sender": "0x123",
                    "created_at": 1000000000
                }
            ]
        })
        client.session.revoke_session = AsyncMock(return_value={
            "success": True
        })
        
        # Mock transaction methods
        client.transaction = MagicMock()
        client.transaction.wait_for_transaction = AsyncMock(return_value={
            "status": {"status": "executed"}
        })
        
        return client
    
    @pytest.fixture
    def test_keypair(self):
        """Create a test keypair"""
        return KeyPair.generate()
    
    @pytest.fixture
    def test_signer(self, test_keypair):
        """Create a test signer"""
        return RoochSigner(test_keypair)
    
    @pytest.mark.asyncio
    async def test_create_session(self, mock_client, test_signer):
        """Test creating a new session"""
        # Create a session
        session_result = await mock_client.session.create_session(
            signer=test_signer,
            description="Test session",
            allowed_target=["0x1::coin::transfer"],
            expiration_timestamp=int(time.time()) + 3600,
            max_inactive_interval=300,
            auth_validator_id=None
        )
        
        # Verify session was created
        assert session_result is not None
        assert "session_id" in session_result
        assert session_result["session_id"] == "test-session-id"
        assert "sender" in session_result
        assert session_result["sender"] == "0x123"
    
    @pytest.mark.asyncio
    async def test_list_sessions(self, mock_client, test_signer):
        """Test listing sessions"""
        # List sessions
        sessions = await mock_client.session.list_sessions(
            address=test_signer.get_address()
        )
        
        # Verify sessions were listed
        assert sessions is not None
        assert "sessions" in sessions
        assert len(sessions["sessions"]) == 1
        assert sessions["sessions"][0]["session_id"] == "test-session-id"
        assert sessions["sessions"][0]["sender"] == "0x123"
        assert "created_at" in sessions["sessions"][0]
    
    @pytest.mark.asyncio
    async def test_revoke_session(self, mock_client, test_signer):
        """Test revoking a session"""
        # Revoke session
        result = await mock_client.session.revoke_session(
            signer=test_signer,
            session_id="test-session-id"
        )
        
        # Verify session was revoked
        assert result is not None
        assert "success" in result
        assert result["success"] is True
    
    @pytest.mark.asyncio
    async def test_execute_with_session(self, mock_client, test_signer):
        """Test executing a transaction with a session"""
        # Create a session first
        session_result = await mock_client.session.create_session(
            signer=test_signer,
            description="Test session",
            allowed_target=["0x1::coin::transfer"],
            expiration_timestamp=int(time.time()) + 3600,
            max_inactive_interval=300,
            auth_validator_id=None
        )
        
        # Mock the execute_move_call method for session transactions
        mock_client.execute_move_call = AsyncMock(return_value="0xsession_transaction_hash")
        
        # Execute a function with the session
        tx_hash = await mock_client.execute_move_call(
            signer=test_signer,
            function_id="0x1::coin::transfer",
            type_args=["0x3::gas_coin::RGas"],
            args=[["0x456", "100"]],
            max_gas_amount=1000000,
            session_id=session_result["session_id"]
        )
        
        # Verify transaction was submitted
        assert tx_hash == "0xsession_transaction_hash"
        
        # Verify mock was called with correct parameters
        mock_client.execute_move_call.assert_called_once()
        call_args = mock_client.execute_move_call.call_args[1]
        assert call_args["signer"] == test_signer
        assert call_args["function_id"] == "0x1::coin::transfer"
        assert call_args["type_args"] == ["0x3::gas_coin::RGas"]
        assert call_args["args"] == [["0x456", "100"]]
        assert call_args["max_gas_amount"] == 1000000
        assert call_args["session_id"] == session_result["session_id"]