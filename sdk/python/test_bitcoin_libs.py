#!/usr/bin/env python3

# Test different Bitcoin libraries for Taproot address generation

def test_bitcoin_libraries():
    # Test public key from Move examples
    pubkey_hex = "034cdb7426f6cebd2e69630c5214fac8dee6a999b43b22907d1d8e4a9363a96a14"
    expected_address = "bc1p72fvqwm9w4wcsd205maky9qejf6dwa6qeku5f5vnu4phpp3vvpws0p2f4g"
    
    print(f"Testing pubkey: {pubkey_hex}")
    print(f"Expected address: {expected_address}")
    print()
    
    # Test bitcoinlib
    try:
        from bitcoinlib.keys import Key
        from bitcoinlib.wallets import Wallet
        print("=== Testing bitcoinlib ===")
        
        # Create key from compressed public key
        key = Key(pubkey_hex)
        print(f"Key type: {key.key_type}")
        print(f"Compressed: {key.compressed}")
        
        # Try to get Taproot address
        if hasattr(key, 'address_taproot'):
            taproot_addr = key.address_taproot()
            print(f"Taproot address: {taproot_addr}")
        else:
            print("Taproot not directly available in this version")
            
    except ImportError as e:
        print(f"bitcoinlib not available: {e}")
    except Exception as e:
        print(f"bitcoinlib error: {e}")
    
    print()
    
    # Test bitcoin library
    try:
        import bitcoin
        print("=== Testing bitcoin library ===")
        PYTHON 应该有 bitcoin 相关的库实现了这个,是不是不需要我们自己实现了.
        # Try to create Taproot address
        if hasattr(bitcoin, 'pubkey_to_p2tr'):
            taproot_addr = bitcoin.pubkey_to_p2tr(pubkey_hex)
            print(f"Taproot address: {taproot_addr}")
        else:
            print("No direct Taproot support found")
            
    except ImportError as e:
        print(f"bitcoin library not available: {e}")
    except Exception as e:
        print(f"bitcoin library error: {e}")

if __name__ == "__main__":
    test_bitcoin_libraries()
