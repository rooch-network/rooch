#!/usr/bin/env python3
# Copyright (c) Rooch Network
# SPDX-License-Identifier: Apache-2.0

"""Test fixtures for Rooch testing"""

import asyncio
import json
import os
import pytest
import socket
from typing import Dict, Any, Callable, List, Iterator

from rooch.client.client import RoochClient
from rooch.transport import RoochEnvironment, RoochTransport
from rooch.crypto.keypair import KeyPair
from rooch.crypto.signer import Signer
from .container_utils import RoochNodeContainer
from rooch.rpc.client import JsonRpcClient


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


def is_port_in_use(port: int) -> bool:
    """Check if a port is in use"""
    with socket.socket(socket.AF_INET, socket.SOCK_STREAM) as s:
        return s.connect_ex(('localhost', port)) == 0


def find_available_port(start_port: int = 6767) -> int:
    """Find an available port starting from start_port"""
    port = start_port
    while is_port_in_use(port):
        port += 1
    return port


@pytest.fixture(scope="session")
def rooch_local_port() -> int:
    """Get an available port for the local Rooch node"""
    return find_available_port()


@pytest.fixture(scope="session")
def rooch_server_url(rooch_local_port: int) -> Iterator[str]:
    """Start a local Rooch node and return the server URL"""
    # Check if we should skip the container (for CI environment that already has a node running)
    if os.environ.get("SKIP_ROOCH_CONTAINER") == "1":
        yield os.environ.get("ROOCH_SERVER_URL", "http://localhost:50051/v1/jsonrpc")
        return

    # Determine if we should use local binary
    local_binary = os.environ.get("ROOCH_LOCAL_BINARY")
    
    # Start container
    container = RoochNodeContainer(
        port=rooch_local_port,
        local_binary_path=local_binary if local_binary else None,
        image=os.environ.get("ROOCH_CONTAINER_IMAGE", "ghcr.io/rooch-network/rooch:main_debug")
    )
    
    server_url = container.start()
    
    try:
        yield server_url
    finally:
        container.stop()


@pytest.fixture(scope="session")
def rooch_client(rooch_server_url: str) -> JsonRpcClient:
    """Create a JsonRpcClient connected to the local Rooch node"""
    return JsonRpcClient(endpoint=rooch_server_url)