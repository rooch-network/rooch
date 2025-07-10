#!/usr/bin/env python3

# Test Bitcoin libraries for Taproot address generation

def test_bitcoin_libraries():
    pubkey_hex = "034cdb7426f6cebd2e69630c5214fac8dee6a999b43b22907d1d8e4a9363a96a14"
    expected_address = "bc1p72fvqwm9w4wcsd205maky9qejf6dwa6qeku5f5vnu4phpp3vvpws0p2f4g"
    
    print(f"Testing pubkey: {pubkey_hex}")
    print(f"Expected address: {expected_address}")
    print()
    
    # Test python-bitcoinlib
    try:
        import bitcoin
        from bitcoin.segwit_addr import encode
        print("=== Testing python-bitcoinlib ===")
        
        pubkey_bytes = bytes.fromhex(pubkey_hex)
        print(f"Pubkey bytes length: {len(pubkey_bytes)}")
        
        # Try to create P2TR address if available
        if hasattr(bitcoin, 'P2TRCoinAddress'):
            print("P2TR support found")
        else:
            print("No P2TR support in this version")
            
    except ImportError as e:
        print(f"python-bitcoinlib not available: {e}")
    except Exception as e:
        print(f"python-bitcoinlib error: {e}")
    
    print()
    
    # Test btclib
    try:
        import btclib
        from btclib import script, addresses
        print("=== Testing btclib ===")
        
        pubkey_bytes = bytes.fromhex(pubkey_hex)
        
        # Try to create Taproot address
        if hasattr(addresses, 'p2tr'):
            # Extract x-coordinate for Taproot
            x_coord = pubkey_bytes[1:] if len(pubkey_bytes) == 33 else pubkey_bytes
            taproot_addr = addresses.p2tr(x_coord, network='mainnet')
            print(f"Taproot address: {taproot_addr}")
            print(f"Matches expected: {taproot_addr == expected_address}")
        else:
            print("No p2tr function found")
            
    except ImportError as e:
        print(f"btclib not available: {e}")
    except Exception as e:
        print(f"btclib error: {e}")
    
    print()
    
    # Test bitcoinlib again with better approach
    try:
        from bitcoinlib.keys import Key
        print("=== Testing bitcoinlib (improved) ===")
        
        # Create key from compressed public key
        key = Key(pubkey_hex, compressed=True)
        print(f"Key created successfully")
        
        # Try different address types
        if hasattr(key, 'address'):
            addr = key.address()
            print(f"Default address: {addr}")
            
        # Try to get all possible addresses
        for attr in dir(key):
            if 'address' in attr.lower() and 'taproot' in attr.lower():
                print(f"Found Taproot method: {attr}")
                
    except ImportError as e:
        print(f"bitcoinlib not available: {e}")
    except Exception as e:
        print(f"bitcoinlib improved error: {e}")

if __name__ == "__main__":
    test_bitcoin_libraries()
