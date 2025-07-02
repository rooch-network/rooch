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
        """Test getting global states (no pagination)"""
        # Test with default parameters (should fail, must provide access_path)
        with pytest.raises(TypeError):
            await rooch_client.get_states()
        # Test with required access_path
        access_path = "/object/0x1"  # Example, adjust as needed
        result = await rooch_client.get_states(access_path)
        assert result is not None
        print(f"States for access path {access_path}: {result}")
        assert isinstance(result, list)
        # Optionally check the structure of the first element if present
        if result:
            assert isinstance(result[0], dict)

    @pytest.mark.asyncio
    async def test_list_states(self, rooch_client: RoochClient):
        """Test list_states with pagination parameters"""
        access_path = "/object"  # Example, adjust as needed
        page = await rooch_client.list_states(access_path, cursor=None, limit=5)
        assert page is not None
        assert "data" in page
        assert isinstance(page["data"], list)
        assert len(page["data"]) <= 5


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
        print(f"Error message: {excinfo.value}")
        # Assert that the error message contains useful information
        assert "ABORTED" in str(excinfo.value)

    @pytest.mark.asyncio
    async def test_invalid_address_format(self, rooch_client: RoochClient):
        """Test using an invalid address format"""
        invalid_address = "not_a_valid_address"
        
        with pytest.raises(Exception) as excinfo:
            await rooch_client.account.get_account(invalid_address)
        print(f"Error message: {excinfo.value}")
        # Assert that the error message contains useful information
        assert "Invalid params" in str(excinfo.value)

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
        )
        
        assert custom_builder is not None
        assert custom_builder.sender_address == address
        assert custom_builder.max_gas_amount == 5_000_000
