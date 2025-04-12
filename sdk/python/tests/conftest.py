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
from unittest.mock import MagicMock, AsyncMock
import pytest_asyncio

from rooch.client.client import RoochClient
from rooch.crypto.keypair import KeyPair
from rooch.crypto.signer import RoochSigner
from .container_utils import RoochNodeContainer
from rooch.rpc.client import JsonRpcClient
from rooch.bcs.serializer import BcsSerializer
from rooch.utils.hex import to_hex


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
                    "coin_type": "0x3::gas_coin::RGas",
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


@pytest_asyncio.fixture(scope="function")
async def rooch_client(rooch_server_url: str) -> RoochClient:
    """Create a RoochClient connected to the local Rooch node for a single test function."""
    client = RoochClient(url_or_env=rooch_server_url)
    async with client:
        yield client


@pytest.fixture(scope="session")
def test_keypair() -> KeyPair:
    """Create a deterministic test keypair"""
    # Use a fixed seed for deterministic keys
    seed = bytes([i % 256 for i in range(32)])
    return KeyPair.from_seed(seed)


@pytest.fixture(scope="session")
def test_signer(test_keypair: KeyPair) -> RoochSigner:
    """Create a deterministic test signer"""
    return RoochSigner(test_keypair)


@pytest_asyncio.fixture(scope="function")
async def setup_integration_test_account(rooch_client: RoochClient, test_signer: RoochSigner):
    """Fixture to ensure the test signer account exists and has Gas for a single test function."""
    print(f"\nAttempting to fund test account: {test_signer.get_address()} for test")
    try:
        # Define the faucet call parameters
        faucet_amount = 100_000_000_000 # Request a significant amount of Gas (e.g., 100 RGas)
        # Removed TransactionArgument, pass raw value
        faucet_arg = faucet_amount 

        # Call the faucet function
        result = await rooch_client.execute_move_call(
            signer=test_signer, 
            function_id="0x3::gas_coin::faucet_entry",
            type_args=[],
            # Pass the raw value in a list
            args=[faucet_arg], 
            max_gas_amount=1_000_000 
        )
        print(f"Faucet call result for {test_signer.get_address()}: {result}")
        # Check if the faucet call itself was successful
        if "execution_info" not in result or result["execution_info"]["status"]["type"] != "executed":
             print(f"Warning: Faucet call might have failed for {test_signer.get_address()}: {result}")
        else:
             print(f"Successfully requested funds for {test_signer.get_address()}")
             # Wait a bit for the state to potentially update after faucet call
             await asyncio.sleep(2)

    except Exception as e:
        # Log the error but don't fail the setup, as the account might already exist
        print(f"Warning: Failed to execute faucet call for {test_signer.get_address()}: {e}")
        print("Proceeding with tests, assuming account might already exist or other tests handle creation.")

    # No yield needed for setup-only fixture if called explicitly


# Local RPC client for integration tests
# This fixture might be redundant now if tests directly use rooch_client
# @pytest.fixture
# def local_client(rooch_client: RoochClient): 
#     """Return the function-scoped client for integration tests"""
#     return rooch_client


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