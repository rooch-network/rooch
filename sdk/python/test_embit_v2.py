#!/usr/bin/env python3

# Test embit for Taproot address generation with proper key objects

def test_embit_taproot():
    pubkey_hex = "034cdb7426f6cebd2e69630c5214fac8dee6a999b43b22907d1d8e4a9363a96a14"
    expected_address = "bc1p72fvqwm9w4wcsd205maky9qejf6dwa6qeku5f5vnu4phpp3vvpws0p2f4g"
    
    print(f"Testing pubkey: {pubkey_hex}")
    print(f"Expected address: {expected_address}")
    print()
    
    try:
        import embit
        from embit import script
        from embit.misc import secp256k1
        print("=== Testing embit with proper key objects ===")
        
        pubkey_bytes = bytes.fromhex(pubkey_hex)
        
        # Create proper PublicKey object
        pubkey_obj = secp256k1.PublicKey.parse(pubkey_bytes)
        print(f"Created PublicKey object: {pubkey_obj}")
        
        # Create Taproot script 
        taproot_script = script.p2tr(pubkey_obj)
        taproot_addr = taproot_script.address()
        print(f"Taproot address: {taproot_addr}")
        print(f"Matches expected: {taproot_addr == expected_address}")
        
        # Also try with the internal pubkey (x-only)
        x_coord = pubkey_bytes[1:] if len(pubkey_bytes) == 33 else pubkey_bytes
        try:
            internal_pubkey = secp256k1.PublicKey.parse(b'\x02' + x_coord)  # Add prefix for parsing
            taproot_script2 = script.p2tr(internal_pubkey)
            taproot_addr2 = taproot_script2.address()
            print(f"Taproot address (method 2): {taproot_addr2}")
        except Exception as e:
            print(f"Method 2 failed: {e}")
            
    except Exception as e:
        print(f"embit error: {e}")
        import traceback
        traceback.print_exc()

if __name__ == "__main__":
    test_embit_taproot()
