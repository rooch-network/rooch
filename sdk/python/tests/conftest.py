#!/usr/bin/env python3
# Copyright (c) Rooch Network
# SPDX-License-Identifier: Apache-2.0

import asyncio
import json
import os
import pytest
from typing import Dict, Any, Callable, List

from rooch.client.client import RoochClient
from rooch.transport import RoochEnvironment, RoochTransport
from rooch.crypto.keypair import KeyPair
from rooch.crypto.signer import Signer


class MockTransport(RoochTransport):
    """Mock transport for testing"""
    
    def __init__(self, responses: Dict[str, Any] = None):
        """Initialize with mock responses
        
        Args:
            responses: Dictionary mapping method names to mock responses
        """
        super().__init__(url="mock://localhost")
        self.responses = responses or {}
        self.requests = []
        
    async def request(self, method: str, params: List = None) -> Any:
        """Mock request implementation
        
        Args:
            method: JSON-RPC method name
            params: Method parameters
            
        Returns:
            Mock response
        
        Raises:
            Exception: If method is not mocked
        """
        self.requests.append({"method": method, "params": params})
        
        if method in self.responses:
            response = self.responses[method]
            if callable(response):
                return response(params)
            return response
            
        raise Exception(f"Mock response not defined for method: {method}")


@pytest.fixture
def mock_responses():
    """Default mock responses for testing"""
    return {
        "rooch_getChainId": 42,
        "rooch_getAccount": {
            "sequence_number": "0",
            "authentication_key": "0x0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef",
            "coin_register_events": {
                "counter": "0",
                "guid": {
                    "id": {
                        "addr": "0x0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef",
                        "creation_num": "0"
                    }
                }
            },
            "key_rotation_events": {
                "counter": "0",
                "guid": {
                    "id": {
                        "addr": "0x0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef",
                        "creation_num": "1"
                    }
                }
            }
        },
        "rooch_getBalances": {
            "balances": [
                {
                    "coin_type": "0x1::coin::ROOCH",
                    "amount": "1000000000"
                }
            ]
        },
        "rooch_submitTransaction": "0x0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef",
        "rooch_getTransactionByHash": {
            "transaction": {
                "hash": "0x0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef",
                "sender": "0x0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef",
                "sequence_number": "0",
                "max_gas_amount": "10000000",
                "gas_unit_price": "1",
                "expiration_timestamp_secs": "1680000000",
            },
            "status": {
                "status": "executed"
            }
        },
        "rooch_executeViewFunction": "1000000000",
    }


@pytest.fixture
def mock_transport(mock_responses):
    """Create a mock transport with default responses"""
    return MockTransport(mock_responses)


@pytest.fixture
def mock_client(mock_transport):
    """Create a client with mock transport"""
    return RoochClient(transport=mock_transport)


@pytest.fixture
def test_keypair():
    """Create a deterministic keypair for testing"""
    # Use a fixed seed for deterministic results
    # In real tests, you might want to use a fixed private key instead
    keypair = KeyPair.from_seed(b"rooch_test_seed_for_deterministic_results")
    return keypair


@pytest.fixture
def test_signer(test_keypair):
    """Create a test signer"""
    return Signer(test_keypair)


# Local RPC client for integration tests
@pytest.fixture
def local_client():
    """Create a client connected to local RPC endpoint
    
    Note: This requires a local Rooch node running
    """
    return RoochClient(RoochEnvironment.LOCAL)


@pytest.fixture
def run_async():
    """Helper to run async tests"""
    def _run_async(coro):
        return asyncio.get_event_loop().run_until_complete(coro)
    return _run_async