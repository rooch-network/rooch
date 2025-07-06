#!/usr/bin/env python3

# Test embit for Taproot address generation

def test_embit_taproot():
    pubkey_hex = "034cdb7426f6cebd2e69630c5214fac8dee6a999b43b22907d1d8e4a9363a96a14"
    expected_address = "bc1p72fvqwm9w4wcsd205maky9qejf6dwa6qeku5f5vnu4phpp3vvpws0p2f4g"
    
    print(f"Testing pubkey: {pubkey_hex}")
    print(f"Expected address: {expected_address}")
    print()
    
    try:
        import embit
        from embit import script
        print("=== Testing embit ===")
        
        pubkey_bytes = bytes.fromhex(pubkey_hex)
        
        # Check available modules
        embit_modules = [attr for attr in dir(embit) if not attr.startswith('_')]
        print(f"Available embit modules: {embit_modules}")
        
        # Try to find Taproot support
        if hasattr(script, 'p2tr'):
            print("Found p2tr in script module")
            x_coord = pubkey_bytes[1:] if len(pubkey_bytes) == 33 else pubkey_bytes
            taproot_script = script.p2tr(x_coord)
            taproot_addr = taproot_script.address()
            print(f"Taproot address: {taproot_addr}")
            print(f"Matches expected: {taproot_addr == expected_address}")
        else:
            print("No p2tr function found in script")
            
        # Check script module functions
        script_functions = [attr for attr in dir(script) if not attr.startswith('_')]
        print(f"Script functions: {script_functions}")
            
    except Exception as e:
        print(f"embit error: {e}")
        import traceback
        traceback.print_exc()

if __name__ == "__main__":
    test_embit_taproot()
