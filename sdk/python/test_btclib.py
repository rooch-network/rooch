#!/usr/bin/env python3

# Test btclib for Taproot address generation

def test_btclib_taproot():
    pubkey_hex = "034cdb7426f6cebd2e69630c5214fac8dee6a999b43b22907d1d8e4a9363a96a14"
    expected_address = "bc1p72fvqwm9w4wcsd205maky9qejf6dwa6qeku5f5vnu4phpp3vvpws0p2f4g"
    
    print(f"Testing pubkey: {pubkey_hex}")
    print(f"Expected address: {expected_address}")
    print()
    
    try:
        from btclib.script import address
        print("=== Testing btclib script.address ===")
        
        pubkey_bytes = bytes.fromhex(pubkey_hex)
        x_coord = pubkey_bytes[1:] if len(pubkey_bytes) == 33 else pubkey_bytes
        
        print(f"X-coordinate: {x_coord.hex()}")
        
        # Check available functions
        addr_functions = [attr for attr in dir(address) if not attr.startswith('_')]
        print(f"Available functions: {addr_functions}")
        
        # Try p2tr if available
        if hasattr(address, 'p2tr'):
            taproot_addr = address.p2tr(x_coord)
            print(f"Taproot address: {taproot_addr}")
            print(f"Matches expected: {taproot_addr == expected_address}")
        else:
            print("No p2tr function found")
            
    except Exception as e:
        print(f"btclib error: {e}")
        import traceback
        traceback.print_exc()

if __name__ == "__main__":
    test_btclib_taproot()
