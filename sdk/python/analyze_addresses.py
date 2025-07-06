#!/usr/bin/env python3

import hashlib

def test_move_expected_addresses():
    """Test the addresses from Move test cases"""
    
    # Test cases from Move
    test_cases = [
        {
            "pubkey": "034cdb7426f6cebd2e69630c5214fac8dee6a999b43b22907d1d8e4a9363a96a14",
            "expected": "bc1p72fvqwm9w4wcsd205maky9qejf6dwa6qeku5f5vnu4phpp3vvpws0p2f4g"
        },
        {
            "pubkey": "4cdb7426f6cebd2e69630c5214fac8dee6a999b43b22907d1d8e4a9363a96a14",  # x-only
            "expected": "bc1p72fvqwm9w4wcsd205maky9qejf6dwa6qeku5f5vnu4phpp3vvpws0p2f4g"
        }
    ]
    
    for i, case in enumerate(test_cases):
        print(f"Test case {i+1}:")
        print(f"  Public key: {case['pubkey']}")
        print(f"  Expected:   {case['expected']}")
        
        # Extract just the raw bytes from expected address for analysis
        expected_addr = case['expected']
        print(f"  Expected length: {len(expected_addr)}")
        
        # For bech32m, the data part starts after "bc1p"
        if expected_addr.startswith("bc1p"):
            data_part = expected_addr[4:]  # Remove "bc1p"
            print(f"  Data part: {data_part}")
            print(f"  Data length: {len(data_part)}")
        
        print()

if __name__ == "__main__":
    test_move_expected_addresses()
