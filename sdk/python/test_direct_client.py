#!/usr/bin/env python3

"""
Direct test of parameter serialization in client execution path
"""

import sys
import os
import asyncio

# Add the project root to the Python path for imports
project_root = os.path.dirname(os.path.abspath(__file__))
sys.path.insert(0, project_root)

async def test_direct_client_execution():
    """Test parameter serialization through the actual client execution path"""
    
    from rooch.client.client import RoochClient
    from rooch.crypto.keypair import KeyPair
    from rooch.crypto.signer import RoochSigner
    from rooch.transactions.tags.type_tags import TypeTag, StructTag
    
    print("🚀 Testing direct client execution path...")
    
    # Create test signer
    sender_kp = KeyPair.generate()
    sender_signer = RoochSigner(sender_kp)
    
    # Create recipient
    recipient_kp = KeyPair.generate()
    recipient_address = str(recipient_kp.get_rooch_address())
    
    print(f"Sender: {sender_signer.get_address()}")
    print(f"Recipient: {recipient_address}")
    
    # Test only building and serializing the transaction, not sending it
    client = RoochClient("http://localhost:6767")
    
    # Build the transaction payload
    gas_coin_struct_tag = StructTag(
        address="0x3", module="gas_coin", name="RGas", type_params=[]
    )
    gas_coin_type_tag = TypeTag.struct(gas_coin_struct_tag)
    type_args_tags = [gas_coin_type_tag]
    
    args = [recipient_address, 1]
    
    print(f"Building transaction with args: {args}")
    
    try:
        # Use the transaction builder to build the payload
        from rooch.transactions.builder import TxBuilder
        tx_builder = TxBuilder()
        
        # This is the exact same path that execute_move_call uses
        payload = tx_builder.build_function_payload(
            function_id="0x3::transfer::transfer_coin",
            ty_args=type_args_tags,
            args=args
        )
        
        print(f"Built payload type: {type(payload)}")
        
        # Now try to serialize it
        from rooch.bcs.serializer import BcsSerializer
        from rooch.transactions.transaction_types import TransactionData, TransactionType
        
        seq_num = await client.get_account_sequence(sender_signer.get_address())
        chain_id = await client.get_chain_id()
        
        tx_data = TransactionData(
            tx_type=TransactionType.MOVE_ACTION,
            tx_arg=payload,
            sequence_number=seq_num,
            max_gas_amount=10_000_000,
            chain_id=chain_id,
            sender=sender_signer.get_address()
        )
        
        print(f"Serializing TransactionData...")
        serializer = BcsSerializer()
        tx_data.serialize(serializer)
        tx_bytes = serializer.output()
        tx_hex = tx_bytes.hex()
        
        print(f"Transaction hex: {tx_hex}")
        
        # Check for type tags in the hex
        target_addr = recipient_address[2:]  # Remove 0x prefix
        if target_addr in tx_hex:
            addr_pos = tx_hex.find(target_addr)
            print(f"Address position: {addr_pos}")
            
            if addr_pos > 2:
                before_addr = tx_hex[addr_pos-2:addr_pos]
                print(f"2 chars before address: {before_addr}")
                
                if before_addr == "04":
                    print("❌ ERROR: Still found ADDRESS type tag (04)!")
                    return False
                elif before_addr == "21":
                    print("⚠️  Found length prefix (21 = 33 bytes)")
                else:
                    print(f"✅ Address prefix: {before_addr}")
        
        # Check for u256 type tag
        amount_hex = "0100000000000000000000000000000000000000000000000000000000000000"
        if amount_hex in tx_hex:
            amount_pos = tx_hex.find(amount_hex)
            print(f"Amount position: {amount_pos}")
            
            if amount_pos > 2:
                before_amount = tx_hex[amount_pos-2:amount_pos]
                print(f"2 chars before amount: {before_amount}")
                
                if before_amount == "0a":
                    print("❌ ERROR: Still found U256 type tag (0a)!")
                    return False
                else:
                    print(f"✅ Amount prefix: {before_amount}")
        
        print("✅ No type tags found in transaction!")
        return True
        
    except Exception as e:
        print(f"❌ Error: {e}")
        import traceback
        traceback.print_exc()
        return False

if __name__ == "__main__":
    try:
        result = asyncio.run(test_direct_client_execution())
        if result:
            print("✅ Test passed!")
            sys.exit(0)
        else:
            print("❌ Test failed!")
            sys.exit(1)
    except Exception as e:
        print(f"❌ Test failed with exception: {e}")
        import traceback
        traceback.print_exc()
        sys.exit(1)
