#!/usr/bin/env python3

"""
Direct integration test for Move function call with u256 fix
"""

import asyncio
import os
import sys

# Add the project root to the Python path for imports
project_root = os.path.dirname(os.path.abspath(__file__))
sys.path.insert(0, project_root)

async def test_move_call_with_u256():
    """Test Move function call with the u256 parameter fix"""
    
    from rooch.client.client import RoochClient
    from rooch.crypto.keypair import KeyPair
    from rooch.crypto.signer import RoochSigner
    from rooch.transactions.types import TypeTag, StructTag
    
    print("🧪 Starting integration test with u256 parameter fix...")
    
    # Use environment variables for test configuration
    private_key_hex = os.getenv("ROOCH_PRIVATE_KEY")
    if not private_key_hex:
        print("❌ ROOCH_PRIVATE_KEY environment variable not set")
        print("Please set ROOCH_PRIVATE_KEY with a valid private key for testing")
        return
    
    rooch_url = os.getenv("ROOCH_EXTERNAL_URL", "http://localhost:6767")
    print(f"Using Rooch URL: {rooch_url}")
    
    try:
        # Create client and signer
        client = RoochClient(rooch_url)
        keypair = KeyPair.from_hex_string(private_key_hex)
        signer = RoochSigner(keypair)
        
        # Get sender address
        sender_address = signer.get_address()
        print(f"Sender address: {sender_address}")
        
        # Create a test recipient
        recipient_kp = KeyPair.generate()
        recipient_address_obj = recipient_kp.get_rooch_address()
        recipient_address_str = str(recipient_address_obj)
        print(f"Recipient address: {recipient_address_str}")
        
        # Test with a small amount
        transfer_amount = 1000000000  # 10 RGAS (with 8 decimal places)
        print(f"Transfer amount: {transfer_amount} (should be u256)")
        
        # Build type arguments
        gas_coin_struct_tag = StructTag(
            address="0x3", module="gas_coin", name="RGas", type_params=[]
        )
        gas_coin_type_tag = TypeTag.struct(gas_coin_struct_tag)
        type_args = [gas_coin_type_tag]
        
        # Build function arguments
        args = [recipient_address_str, transfer_amount]
        
        print(f"Function ID: 0x3::transfer::transfer_coin")
        print(f"Type args: {type_args}")
        print(f"Args: {args}")
        
        # Execute the move call
        print("\n🚀 Executing move call...")
        result = await client.execute_move_call(
            signer=signer,
            function_id="0x3::transfer::transfer_coin",
            type_args=type_args,
            args=args,
            max_gas_amount=10_000_000
        )
        
        print(f"\n✅ Move call executed successfully!")
        print(f"Result: {result}")
        
        if isinstance(result, dict):
            if "execution_info" in result:
                print(f"\nExecution info:")
                execution_info = result["execution_info"]
                for key, value in execution_info.items():
                    print(f"  {key}: {value}")
                
                # Check if transaction was successful
                if "status" in execution_info:
                    status = execution_info["status"]
                    if status.get("type") == "executed":
                        print("🎉 Transaction executed successfully!")
                    else:
                        print(f"⚠️  Transaction status: {status}")
                        
            if "error" in result:
                print(f"\n❌ Error in result: {result['error']}")
                
    except Exception as e:
        print(f"\n❌ Test failed with exception:")
        print(f"Exception type: {type(e)}")
        print(f"Exception message: {str(e)}")
        if hasattr(e, '__dict__'):
            print(f"Exception attributes: {e.__dict__}")
        
        # Print detailed error information
        import traceback
        print(f"\nFull traceback:")
        traceback.print_exc()

if __name__ == "__main__":
    asyncio.run(test_move_call_with_u256())
