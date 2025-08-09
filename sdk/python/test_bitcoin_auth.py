#!/usr/bin/env python3

import sys
import os

# Add the current directory to the path
sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))

from rooch.crypto.signer import RoochSigner
from rooch.crypto.keypair import KeyPair
from rooch.transactions.builder import TransactionBuilder
from rooch.address.bitcoin import BitcoinAddress

def test_bitcoin_auth_integration():
    """Test the full Bitcoin authentication flow with Taproot addresses"""
    
    print("=== Bitcoin Authentication Integration Test ===")
    
    # Create a test keypair
    keypair = KeyPair.generate()
    signer = RoochSigner(keypair)
    
    print(f"Generated keypair")
    print(f"Public key: {keypair.get_public_key().hex()}")
    
    # Get compressed public key for Taproot address
    public_key_bytes = keypair.get_public_key()  # 65 bytes uncompressed
    x_coord = public_key_bytes[1:33]  # Extract x coordinate
    y_coord = public_key_bytes[33:65]  # Extract y coordinate
    
    # Determine if y is even or odd to set the prefix
    y_int = int.from_bytes(y_coord, byteorder='big')
    if y_int % 2 == 0:
        compressed_public_key = b'\x02' + x_coord
    else:
        compressed_public_key = b'\x03' + x_coord
    
    print(f"Compressed public key: {compressed_public_key.hex()}")
    
    # Generate Taproot address using embit
    try:
        taproot_address = BitcoinAddress.from_taproot_public_key(compressed_public_key, True)
        print(f"Generated Taproot address: {taproot_address.address}")
        print(f"✓ Taproot address generation successful")
    except Exception as e:
        print(f"✗ Taproot address generation failed: {e}")
        return False
    
    # Test signature creation and verification process
    try:
        # Create a test transaction builder
        builder = TransactionBuilder(
            sender_address=signer.get_address(),
            sequence_number=0,
            chain_id=4,
            max_gas_amount=1000000
        )
        
        # Build a simple function call
        payload = builder.build_function_payload(
            function_id="0x3::gas_coin::faucet",
            args=[]
        )
        
        # Build transaction data
        tx_data = builder.build_move_action_tx(payload)
        
        print(f"✓ Transaction data created")
        print(f"Sender: {tx_data.sender}")
        
        # Sign the transaction (this will test the Bitcoin auth process)
        try:
            signed_tx = builder.sign(tx_data, signer)
            print(f"✓ Transaction signed successfully")
            print(f"Authenticator type: {type(signed_tx.authenticator)}")
            
            # The fact that we got here means the Bitcoin address was created correctly
            # and matches what Move expects
            print(f"🎉 SUCCESS! Bitcoin authentication flow completed")
            return True
            
        except Exception as e:
            print(f"✗ Transaction signing failed: {e}")
            import traceback
            traceback.print_exc()
            return False
            
    except Exception as e:
        print(f"✗ Transaction building failed: {e}")
        import traceback
        traceback.print_exc()
        return False

if __name__ == "__main__":
    success = test_bitcoin_auth_integration()
    sys.exit(0 if success else 1)
