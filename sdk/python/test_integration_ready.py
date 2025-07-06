#!/usr/bin/env python3

import sys
import os

# Add the current directory to the path
sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))

from rooch.crypto.signer import RoochSigner
from rooch.crypto.keypair import KeyPair
from rooch.transactions.builder import TransactionBuilder
from rooch.address.bitcoin import BitcoinAddress
from rooch.transactions.types import TypeTag, StructTag

def test_integration_ready():
    """Test that integration tests should now work with Bitcoin authentication"""
    
    print("=== Integration Test Readiness Check ===")
    print("Testing the flow similar to test_client_integration.py")
    
    # Simulate the integration test setup
    test_keypair = KeyPair.generate()
    test_signer = RoochSigner(test_keypair)
    
    print(f"\n1. Test signer created")
    print(f"   Address: {test_signer.get_address()}")
    
    # Create a recipient (like in the integration test)
    recipient_kp = KeyPair.generate()
    recipient_address_obj = recipient_kp.get_rooch_address()
    recipient_address_str = str(recipient_address_obj)
    amount = 100
    move_call_args = [recipient_address_str, amount]
    
    print(f"\n2. Transfer parameters prepared")
    print(f"   Recipient: {recipient_address_str}")
    print(f"   Amount: {amount}")
    
    # Set up the type args (like in integration test)
    gas_coin_struct_tag = StructTag(
        address="0x3", module="gas_coin", name="GasCoin", type_params=[]
    )
    gas_coin_type_tag = TypeTag.struct(gas_coin_struct_tag)
    type_args_tags = [gas_coin_type_tag]
    
    # Create transaction builder (like client.get_transaction_builder)
    builder = TransactionBuilder(
        sender_address=test_signer.get_address(),  # This uses KeyPair-derived address
        sequence_number=0,
        chain_id=4,
        max_gas_amount=10_000_000
    )
    
    print(f"\n3. Transaction builder created")
    print(f"   Initial sender: {builder.sender_address}")
    
    # Build the function payload (like execute_move_call)
    try:
        payload = builder.build_function_payload(
            function_id="0x3::transfer::transfer_coin",
            ty_args=type_args_tags,
            args=move_call_args
        )
        print(f"   ✓ Function payload built")
        
        # Build transaction data
        tx_data = builder.build_move_action_tx(payload)
        print(f"   ✓ Transaction data built")
        print(f"   TX sender: {tx_data.sender}")
        
        # Sign the transaction (this should auto-correct the sender)
        signed_tx = builder.sign(tx_data, test_signer)
        final_sender = str(signed_tx.tx_data.sender)
        print(f"   ✓ Transaction signed")
        print(f"   Final sender: {final_sender}")
        
        # Check if sender was corrected
        original_sender = test_signer.get_address()
        if final_sender != original_sender:
            print(f"   ✓ Sender auto-corrected for Bitcoin auth")
            print(f"     Original: {original_sender}")
            print(f"     Bitcoin:  {final_sender}")
        else:
            print(f"   ⚠ Sender not changed (unexpected)")
        
        # Verify the Bitcoin address chain
        public_key_bytes = test_keypair.get_public_key()
        x_coord = public_key_bytes[1:33]
        y_coord = public_key_bytes[33:65]
        y_int = int.from_bytes(y_coord, byteorder='big')
        compressed_key = (b'\x02' if y_int % 2 == 0 else b'\x03') + x_coord
        
        # Generate Bitcoin address
        btc_addr = BitcoinAddress.from_taproot_public_key(compressed_key, True)
        btc_rooch_addr = btc_addr.to_rooch_address()
        
        print(f"\n4. Bitcoin authentication chain verified")
        print(f"   Bitcoin address: {btc_addr.address}")
        print(f"   Bitcoin->Rooch:  {btc_rooch_addr}")
        print(f"   Matches final:   {btc_rooch_addr == final_sender}")
        
        if btc_rooch_addr == final_sender:
            print(f"\n🎉 INTEGRATION TEST READY!")
            print(f"   ✓ Address mapping issue fixed")
            print(f"   ✓ Bitcoin authentication should work")
            print(f"   ✓ No more error 1010 expected")
            return True
        else:
            print(f"\n❌ Address mismatch still exists")
            return False
            
    except Exception as e:
        print(f"\n❌ Transaction building/signing failed: {e}")
        import traceback
        traceback.print_exc()
        return False

if __name__ == "__main__":
    print("Final verification: Integration tests should now work with Bitcoin authentication")
    print("Issue: sender 和 btc地址得到的 rooch 地址不一致")
    print("Fix: TransactionBuilder automatically corrects sender address for Bitcoin auth")
    
    success = test_integration_ready()
    
    if success:
        print(f"\n✅ VERIFICATION COMPLETE")
        print("Integration tests (test_client_integration.py) should now pass!")
        print("The address consistency issue has been resolved.")
        sys.exit(0)
    else:
        print(f"\n❌ VERIFICATION FAILED")
        print("There may still be issues with the integration tests.")
        sys.exit(1)
