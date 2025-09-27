#!/usr/bin/env python3

# Test embit with correct PublicKey class

def test_embit_taproot():
    pubkey_hex = "034cdb7426f6cebd2e69630c5214fac8dee6a999b43b22907d1d8e4a9363a96a14"
    expected_address = "bc1p72fvqwm9w4wcsd205maky9qejf6dwa6qeku5f5vnu4phpp3vvpws0p2f4g"
    
    print(f"Testing pubkey: {pubkey_hex}")
    print(f"Expected address: {expected_address}")
    print()
    
    try:
        from embit.ec import PublicKey
        from embit import script
        print("=== Testing embit with correct PublicKey ===")
        
        pubkey_bytes = bytes.fromhex(pubkey_hex)
        
        # Create PublicKey object
        pubkey_obj = PublicKey.parse(pubkey_bytes)
        print(f"Created PublicKey object: {pubkey_obj}")
        
        # Create Taproot script 
        taproot_script = script.p2tr(pubkey_obj)
        taproot_addr = taproot_script.address()
        print(f"Taproot address: {taproot_addr}")
        print(f"Matches expected: {taproot_addr == expected_address}")
        
        if taproot_addr == expected_address:
            print("🎉 SUCCESS! embit generated the correct Taproot address!")
            return True
        else:
            print("❌ Address mismatch")
            return False
            
    except Exception as e:
        print(f"embit error: {e}")
        import traceback
        traceback.print_exc()
        return False

if __name__ == "__main__":
    test_embit_taproot()
