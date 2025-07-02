import pytest
import pytest_asyncio
import asyncio
from typing import Dict, Any, List

from rooch.client.client import RoochClient
from rooch.crypto.keypair import KeyPair
from rooch.crypto.signer import Signer
from rooch.address.rooch import RoochAddress
from rooch.client.types.json_rpc import JsonRpcResponse

# Mark all tests in this module as integration tests
pytestmark = pytest.mark.integration

class TestRPCMethods:
    """Tests for comprehensive coverage of Rooch RPC methods"""

    @pytest.mark.asyncio
    async def test_get_states(self, rooch_client: RoochClient):
        """Test getting global states with pagination"""
        # Test with default parameters
        result = await rooch_client.get_states()
        assert result is not None
        assert "cursor" in result
        assert "data" in result
        assert isinstance(result["data"], list)
        
        # Test with custom pagination
        custom_result = await rooch_client.get_states(cursor=0, limit=5)
        assert custom_result is not None
        assert "cursor" in custom_result
        assert "data" in custom_result
        assert isinstance(custom_result["data"], list)
        assert len(custom_result["data"]) <= 5

    @pytest.mark.asyncio
    async def test_get_states_by_prefix(self, rooch_client: RoochClient):
        """Test getting states by prefix with pagination"""
        # Use a common prefix that should exist
        prefix = "0x1"
        
        result = await rooch_client.get_states_by_prefix(prefix)
        assert result is not None
        assert "cursor" in result
        assert "data" in result
        assert isinstance(result["data"], list)
        
        # Test with custom pagination
        custom_result = await rooch_client.get_states_by_prefix(prefix, cursor=0, limit=3)
        assert custom_result is not None
        assert "cursor" in custom_result
        assert "data" in custom_result
        assert isinstance(custom_result["data"], list)
        assert len(custom_result["data"]) <= 3

    @pytest.mark.asyncio
    async def test_get_block_info_by_height(self, rooch_client: RoochClient):
        """Test getting block info by height"""
        # Get a block from a height that should exist (usually 1 is safe)
        height = 1
        block_info = await rooch_client.get_block_info_by_height(height)
        
        assert block_info is not None
        # Check for expected block info fields (adjust based on actual response structure)
        assert "height" in block_info or "timestamp" in block_info


class TestErrorHandling:
    """Tests for error handling in the SDK"""
    
    @pytest.mark.asyncio
    async def test_invalid_function_id(self, rooch_client: RoochClient, test_signer: Signer):
        """Test calling an invalid function ID"""
        invalid_function_id = "0x1::nonexistent_module::nonexistent_function"
        
        with pytest.raises(Exception) as excinfo:
            await rooch_client.execute_move_call(
                signer=test_signer,
                function_id=invalid_function_id,
                type_args=[],
                args=[]
            )
        
        # Assert that the error message contains useful information
        assert "nonexistent_module" in str(excinfo.value) or "function" in str(excinfo.value)

    @pytest.mark.asyncio
    async def test_invalid_address_format(self, rooch_client: RoochClient):
        """Test using an invalid address format"""
        invalid_address = "not_a_valid_address"
        
        with pytest.raises(Exception) as excinfo:
            await rooch_client.account.get_account(invalid_address)
        
        # Assert that the error message contains useful information
        assert "address" in str(excinfo.value) or "format" in str(excinfo.value)

    @pytest.mark.asyncio
    async def test_get_nonexistent_block(self, rooch_client: RoochClient):
        """Test getting a block that doesn't exist"""
        # Use a very high block height that shouldn't exist
        very_high_height = 999999999
        
        # This should either return None or raise an appropriate exception
        try:
            block = await rooch_client.get_block_by_height(very_high_height)
            # If it returns None or an empty object, that's valid behavior
            assert block is None or block == {} or "error" in block
        except Exception as e:
            # If it raises an exception, ensure it's appropriate
            assert "not found" in str(e) or "invalid" in str(e) or "height" in str(e)


class TestAdvancedTransactions:
    """Tests for advanced transaction functionality"""

    @pytest.mark.asyncio
    async def test_transaction_builder_configuration(self, rooch_client: RoochClient, test_signer: Signer):
        """Test creating a transaction builder with different configurations"""
        address = test_signer.get_address()
        
        # Test with default configuration
        builder = await rooch_client.get_transaction_builder(address)
        assert builder is not None
        assert builder.sender_address == address
        
        # Test with custom configuration
        custom_builder = await rooch_client.get_transaction_builder(
            sender_address=address,
            max_gas_amount=5_000_000,
            gas_unit_price=2,
            expiration_delta_secs=300
        )
        
        assert custom_builder is not None
        assert custom_builder.sender_address == address
        assert custom_builder.max_gas_amount == 5_000_000
        assert custom_builder.gas_unit_price == 2
        assert custom_builder.expiration_delta_secs == 300

    @pytest.mark.asyncio
    async def test_module_publishing_validation(self, rooch_client: RoochClient, test_signer: Signer):
        """Test module publishing input validation"""
        # Test with empty bytecode (should fail gracefully)
        empty_bytes = b""
        
        with pytest.raises(Exception) as excinfo:
            await rooch_client.publish_module(
                signer=test_signer,
                module_bytes=empty_bytes
            )
        
        # Check that the error message is helpful
        assert "empty" in str(excinfo.value) or "invalid" in str(excinfo.value) or "module" in str(excinfo.value)
