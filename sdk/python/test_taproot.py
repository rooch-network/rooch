#!/usr/bin/env python3

import sys
import os

# Add the current directory to the path
sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))

from rooch.address.bitcoin import BitcoinAddress, BitcoinNetworkType

def test_taproot_address():
    # Test public key from Move examples
    test_public_key = bytes.fromhex("034cdb7426f6cebd2e69630c5214fac8dee6a999b43b22907d1d8e4a9363a96a14")
    expected_address = "bc1p72fvqwm9w4wcsd205maky9qejf6dwa6qeku5f5vnu4phpp3vvpws0p2f4g"
    
    print(f"Test public key: {test_public_key.hex()}")
    print(f"Public key length: {len(test_public_key)}")
    print(f"Expected address: {expected_address}")
    
    try:
        # Generate Taproot address
        taproot_address = BitcoinAddress.from_taproot_public_key(
            test_public_key, 
            True  # mainnet = True
        )
        
        print(f"Generated Taproot address: {taproot_address.address}")
        print(f"Network: mainnet")
        
        # Compare with expected
        if taproot_address.address == expected_address:
            print("✓ Generated address matches expected address!")
        else:
            print("✗ Generated address does not match expected address")
        
        # Validate the address
        is_valid = taproot_address.validate_address(taproot_address.address)
        print(f"Is valid: {is_valid}")
        
        # Check if it starts with bc1p
        if taproot_address.address.startswith("bc1p"):
            print("✓ Address has correct Taproot prefix")
        else:
            print("✗ Address does not have correct Taproot prefix")
            
    except Exception as e:
        print(f"Error generating Taproot address: {e}")
        import traceback
        traceback.print_exc()

if __name__ == "__main__":
    test_taproot_address()
