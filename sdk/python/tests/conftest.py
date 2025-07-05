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
        faucet_amount = 100_00_000_000 # Request a significant amount of Gas
        # Removed TransactionArgument, pass raw value
        faucet_arg = faucet_amount 

        # Call the faucet function
        print(f"Attempting faucet call with signer address: {test_signer.get_address()}")
        print(f"Faucet amount: {faucet_arg}")
        try:
            result = await rooch_client.execute_move_call(
                signer=test_signer, 
                function_id="0x3::gas_coin::faucet_entry",
                type_args=[],
                args=[faucet_arg], 
                max_gas_amount=100_000_000 
            )
            print(f"Faucet call result for {test_signer.get_address()}: {result}")
            # Check if the faucet call itself was successful
            if "execution_info" not in result or result["execution_info"]["status"]["type"] != "executed":
                print(f"Warning: Faucet call might have failed for {test_signer.get_address()}: {result}")
                if "error" in result:
                    print(f"Error details: {result['error']}")
                    print(f"Error type: {type(result['error'])}")
                    if hasattr(result['error'], '__dict__'):
                        print(f"Error attributes: {result['error'].__dict__}")
                if "execution_info" in result:
                    print(f"Execution info: {result['execution_info']}")
                    if "error" in result["execution_info"]:
                        print(f"Execution error: {result['execution_info']['error']}")
            else:
                print(f"Successfully requested funds for {test_signer.get_address()}")
                # Wait a bit for the state to potentially update after faucet call
                await asyncio.sleep(2)

        except Exception as e:
            # Log the error but don't fail the setup, as the account might already exist
            print(f"Warning: Failed to execute faucet call for {test_signer.get_address()}: {e}")
            print("Proceeding with tests, assuming account might already exist or other tests handle creation.")

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
    """
    Start a local Rooch node for testing or use an external one if specified.

    This fixture provides a Rooch server URL for integration tests.
    It checks for the `ROOCH_EXTERNAL_URL` environment variable.
    - If set, it yields that URL, allowing tests to run against a pre-existing Rooch node.
    - If not set, it starts a new, temporary Rooch node (either from a local binary
      if `ROOCH_LOCAL_BINARY` is set, or from a Docker container) and yields its URL.
    The temporary node is automatically stopped at the end of the test session.
    """
    # Check if an external Rooch node URL is provided
    external_url = os.environ.get("ROOCH_EXTERNAL_URL")
    if external_url:
        print(f"\nUsing external Rooch node at: {external_url}")
        yield external_url
        return

    # Determine if we should use a local binary instead of a container
    local_binary = os.environ.get("ROOCH_LOCAL_BINARY")
    if local_binary:
        print(f"\nUsing local Rooch binary from: {local_binary}")
    else:
        print("\nStarting Rooch node from Docker container...")

    # Start a managed Rooch node (container or local binary)
    container = RoochNodeContainer(
        port=rooch_local_port,
        local_binary_path=local_binary if local_binary else None,
        image=os.environ.get("ROOCH_CONTAINER_IMAGE", "ghcr.io/rooch-network/rooch:main_debug")
    )
    
    try:
        server_url = container.start()
        print(f"Rooch node started at: {server_url}")
        yield server_url
    finally:
        print("\nStopping Rooch node...")
        container.stop()
        print("Rooch node stopped.")