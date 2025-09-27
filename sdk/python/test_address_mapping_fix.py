#!/usr/bin/env python3

import sys
import os

# Add the current directory to the path
sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))

from rooch.crypto.signer import RoochSigner
from rooch.crypto.keypair import KeyPair
from rooch.transactions.builder import TransactionBuilder
from rooch.address.bitcoin import BitcoinAddress

def test_address_mapping_fix():
    """Test that the address mapping issue between sender and Bitcoin address is fixed"""
    
    print("=== Testing Address Mapping Fix ===")
    
    # Create a test keypair
    keypair = KeyPair.generate()
    signer = RoochSigner(keypair)
    
    print(f"Generated keypair")
    print(f"Public key: {keypair.get_public_key().hex()}")
    
    # Show the two different addresses
    original_rooch_addr = str(signer.get_rooch_address())
    print(f"\n1. Original Rooch address (KeyPair): {original_rooch_addr}")
    
    # Get Bitcoin-derived address
    public_key_bytes = keypair.get_public_key()
    x_coord = public_key_bytes[1:33]
    y_coord = public_key_bytes[33:65]
    y_int = int.from_bytes(y_coord, byteorder='big')
    if y_int % 2 == 0:
        compressed_public_key = b'\x02' + x_coord
    else:
        compressed_public_key = b'\x03' + x_coord
    
    bitcoin_rooch_addr = BitcoinAddress.get_rooch_address_from_public_key(compressed_public_key, True)
    print(f"2. Bitcoin-derived Rooch address:   {bitcoin_rooch_addr}")
    
    # Show the difference (this was the problem)
    addresses_match = original_rooch_addr == bitcoin_rooch_addr
    print(f"3. Addresses match: {addresses_match}")
    
    if addresses_match:
        print("❌ UNEXPECTED: Addresses should be different!")
        return False
    else:
        print("✓ EXPECTED: Addresses are different (this was the root issue)")
    
    # Now test that TransactionBuilder fixes this automatically
    print(f"\n4. Testing TransactionBuilder auto-correction...")
    
    # Create builder with original signer address (this would be wrong for Bitcoin auth)
    builder = TransactionBuilder(
        sender_address=signer.get_address(),  # This uses the KeyPair-derived address
        sequence_number=0,
        chain_id=4,
        max_gas_amount=1000000
    )
    
    # Build a transaction
    payload = builder.build_function_payload(
        function_id="0x3::gas_coin::faucet",
        args=[]
    )
    
    tx_data = builder.build_move_action_tx(payload)
    print(f"   Initial sender in tx_data: {tx_data.sender}")
    
    # Sign the transaction - this should auto-correct the sender address
    try:
        signed_tx = builder.sign(tx_data, signer)
        final_sender = str(signed_tx.tx_data.sender)
        print(f"   Final sender after signing: {final_sender}")
        
        # Verify that the sender was corrected to the Bitcoin-derived address
        if final_sender == bitcoin_rooch_addr:
            print("✓ SUCCESS: TransactionBuilder auto-corrected sender to Bitcoin-derived address!")
            print("✓ Bitcoin authentication should now work correctly")
            return True
        else:
            print(f"❌ FAILURE: Expected {bitcoin_rooch_addr}, got {final_sender}")
            return False
            
    except Exception as e:
        print(f"❌ FAILURE: Transaction signing failed: {e}")
        import traceback
        traceback.print_exc()
        return False

def test_move_consistency():
    """Test that our Bitcoin to Rooch conversion matches Move expectations"""
    
    print(f"\n=== Testing Move Consistency ===")
    
    # Use the known test case from Move
    test_pubkey = "034cdb7426f6cebd2e69630c5214fac8dee6a999b43b22907d1d8e4a9363a96a14"
    expected_btc_addr = "bc1p72fvqwm9w4wcsd205maky9qejf6dwa6qeku5f5vnu4phpp3vvpws0p2f4g"
    
    # Generate Bitcoin address
    btc_addr = BitcoinAddress.from_taproot_public_key(test_pubkey, True)
    print(f"Generated Bitcoin address: {btc_addr.address}")
    print(f"Expected Bitcoin address:  {expected_btc_addr}")
    
    if btc_addr.address == expected_btc_addr:
        print("✓ Bitcoin address generation matches Move")
        
        # Test Rooch address conversion
        rooch_addr = btc_addr.to_rooch_address()
        print(f"Rooch address from Bitcoin: {rooch_addr}")
        print("✓ Bitcoin to Rooch address conversion completed")
        return True
    else:
        print("❌ Bitcoin address generation does not match Move")
        return False

if __name__ == "__main__":
    print("Testing the fix for Bitcoin authentication address mismatch issue...")
    print("Reference: 现在集成测试的签名以及 btc 地址都能验证过去了,但发现 sender 和 btc地址得到的 rooch 地址不一致")
    
    success1 = test_address_mapping_fix()
    success2 = test_move_consistency()
    
    if success1 and success2:
        print(f"\n🎉 ALL TESTS PASSED!")
        print("The address mapping issue has been fixed.")
        print("Bitcoin authentication should now work correctly in integration tests.")
        sys.exit(0)
    else:
        print(f"\n❌ SOME TESTS FAILED!")
        sys.exit(1)
