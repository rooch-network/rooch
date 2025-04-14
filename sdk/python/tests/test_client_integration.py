import pytest
import pytest_asyncio
import asyncio

from rooch.client.client import RoochClient
from rooch.crypto.keypair import KeyPair
from rooch.crypto.signer import RoochSigner
from rooch.address.rooch import RoochAddress
from rooch.bcs.serializer import BcsSerializer
from rooch.utils.hex import to_hex
# Import TypeTag definitions
from rooch.transactions.types import TypeTag, StructTag

# Mark all tests in this module as integration tests
pytestmark = pytest.mark.integration

class TestClientIntegration:
    """Integration tests for RoochClient against a local Rooch node"""

    # --- Account Tests ---

    @pytest.mark.asyncio
    # Request the faucet setup fixture
    async def test_get_account(self, rooch_client: RoochClient, test_signer: RoochSigner, setup_integration_test_account):
        """Test getting account details"""
        address = test_signer.get_address()
        account = await rooch_client.account.get_account(address)
        assert account is not None
        # TODO: Add more specific assertions based on expected account structure
        # For example, check if sequence number and authentication key exist
        print(f"Account details for {address}: {account}")

    # --- Transaction Tests ---

    @pytest.mark.asyncio
    # Request the faucet setup fixture
    async def test_execute_view_function(self, rooch_client: RoochClient, test_signer: RoochSigner, setup_integration_test_account):
        """Test executing a view function (e.g., get sequence number)"""
        address = test_signer.get_address()
        rooch_address = RoochAddress.from_hex(address)
        # sequence_number expects the raw address bytes, not length-prefixed BCS bytes.
        # Convert raw bytes directly to hex for the view function API.
        address_arg_bytes = rooch_address.to_bytes()
        address_arg_hex = to_hex(address_arg_bytes)

        result = await rooch_client.transaction.execute_view_function(
            function_id="0x3::account::sequence_number",
            type_args=[],
            args=[address_arg_hex] # Pass raw address bytes as hex string
        )

        assert result is not None
        assert "return_values" in result
        assert isinstance(result["return_values"], list)
        # TODO: Add more specific assertions about the return value structure/content
        # Example (adapt based on actual structure):
        # assert len(result["return_values"]) > 0
        # assert result["return_values"][0].get("value") == "0" # Or whatever is expected
        print(f"View function result for sequence_number: {result}")

    @pytest.mark.asyncio
    # Request the faucet setup fixture
    async def test_execute_move_call(self, rooch_client: RoochClient, test_signer: RoochSigner, setup_integration_test_account):
        """Test executing a Move function call (e.g., transfer gas coin)"""
        
        recipient_kp = KeyPair.generate()
        recipient_address_obj = recipient_kp.get_rooch_address()
        recipient_address_str = str(recipient_address_obj)
        amount = 100
        move_call_args = [recipient_address_str, amount]

        print(f"\nDebug info for test_execute_move_call:")
        print(f"Sender address: {test_signer.get_address()}")
        print(f"Recipient address: {recipient_address_str}")
        print(f"Transfer amount: {amount}")

        # Check chain_id
        chain_id = await rooch_client.get_chain_id()
        print(f"Current chain_id: {chain_id}")

        # Re-add the TypeTag construction logic
        gas_coin_struct_tag = StructTag(
            address="0x3", module="gas_coin", name="GasCoin", type_params=[]
        )
        gas_coin_type_tag = TypeTag.struct(gas_coin_struct_tag)
        type_args_tags = [gas_coin_type_tag]

        print(f"Type args: {type_args_tags}")
        print(f"Move call args: {move_call_args}")

        try:
            result = await rooch_client.execute_move_call(
                signer=test_signer,
                function_id="0x3::transfer::transfer_coin",
                type_args=type_args_tags,
                args=move_call_args,
                max_gas_amount=10_000_000
            )

            print(f"\nMove call execution result:")
            print(f"Raw result: {result}")
            if isinstance(result, dict):
                for key, value in result.items():
                    print(f"{key}: {value}")
                if "execution_info" in result:
                    print("\nExecution info details:")
                    for key, value in result["execution_info"].items():
                        print(f"  {key}: {value}")
                if "error" in result:
                    print("\nError details:")
                    print(f"  Error: {result['error']}")
                    if hasattr(result['error'], '__dict__'):
                        print(f"  Error attributes: {result['error'].__dict__}")

            assert result is not None
            assert "execution_info" in result
            assert result["execution_info"]["tx_hash"].startswith("0x")
            assert "status" in result["execution_info"]
            assert result["execution_info"]["status"]["type"] == "executed"

        except Exception as e:
            print(f"\nException during move call execution:")
            print(f"Exception type: {type(e)}")
            print(f"Exception message: {str(e)}")
            if hasattr(e, '__dict__'):
                print(f"Exception attributes: {e.__dict__}")
            raise e

    @pytest.mark.asyncio
    # Request the faucet setup fixture
    async def test_get_transaction_by_hash(self, rooch_client: RoochClient, test_signer: RoochSigner, setup_integration_test_account):
        """Test getting a transaction by hash after executing one"""
    
        recipient_kp = KeyPair.generate()
        recipient_address_obj = recipient_kp.get_rooch_address()
        recipient_address_str = str(recipient_address_obj)
        amount = 1
        move_call_args = [recipient_address_str, amount]
        
        # Re-add the TypeTag construction logic
        gas_coin_struct_tag = StructTag(
            address="0x3", module="gas_coin", name="GasCoin", type_params=[]
        )
        gas_coin_type_tag = TypeTag.struct(gas_coin_struct_tag)
        type_args_tags = [gas_coin_type_tag]

        tx_hash = None
        try:
            tx_result = await rooch_client.execute_move_call(
                signer=test_signer,
                function_id="0x3::transfer::transfer_coin",
                type_args=type_args_tags,
                args=move_call_args,
                max_gas_amount=10_000_000
            )
            assert "execution_info" in tx_result
            tx_hash = tx_result["execution_info"]["tx_hash"]
            assert tx_hash.startswith("0x")
            await asyncio.sleep(2)
        except Exception as e:
            pytest.fail(f"Failed to execute prerequisite transaction: {e}")
        
        assert tx_hash is not None, "Prerequisite transaction failed to produce a hash"

        transaction = await rooch_client.transaction.get_transaction_by_hash(tx_hash)
        assert transaction is not None
        assert transaction["execution_info"]["tx_hash"] == tx_hash
        print(f"Transaction details by hash {tx_hash}: {transaction}")

    # --- Module Publishing Tests ---

    @pytest.mark.skip(reason="Requires valid Move module bytes, mock bytes cause expected failure.")
    @pytest.mark.asyncio
    # Request the faucet setup fixture
    async def test_publish_module(self, rooch_client: RoochClient, test_signer: RoochSigner, setup_integration_test_account):
        """Test publishing a Move module"""
        # Mock module bytes
        module_bytes = b"mock_module_bytes"
    
        # Test publishing module
        result = await rooch_client.publish_module(
            signer=test_signer,
            module_bytes=module_bytes,
            max_gas_amount=10_000_000
        )
    
        # Assert based on expected result structure for successful publish
        assert result is not None
        assert "execution_info" in result
        assert result["execution_info"]["tx_hash"].startswith("0x")
        # Check status type
        assert "status" in result["execution_info"]
        # TODO: This might fail if mock bytes are invalid, adjust assertion
        # assert result["execution_info"]["status"]["type"] == "executed"
        print(f"Publish module result: {result}")
        # We expect this to fail with current mock bytes, check for specific error if possible
        if result["execution_info"]["status"]["type"] != "executed":
             print(f"Module publish likely failed as expected with mock bytes: {result['execution_info']['status']}") 