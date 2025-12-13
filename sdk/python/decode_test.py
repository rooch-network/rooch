#!/usr/bin/env python3

def decode_bech32m(bech32_str):
    """Decode a bech32m string to get the witness version and data"""
    CHARSET = "qpzry9x8gf2tvdw0s3jn54khce6mua7l"
    
    if '1' not in bech32_str:
        return None, None, None
    
    hrp, data_part = bech32_str.split('1', 1)
    
    if len(data_part) < 6:  # Need at least 6 chars for checksum
        return None, None, None
    
    # Convert from bech32 charset to 5-bit values
    try:
        decoded = [CHARSET.index(c) for c in data_part]
    except ValueError:
        return None, None, None
    
    # Extract witness version (first value)
    witness_version = decoded[0]
    
    # Extract data (everything except last 6 chars which are checksum)
    data_5bit = decoded[1:-6]
    
    # Convert from 5-bit to 8-bit
    def convertbits(data, frombits, tobits, pad=False):
        acc = 0
        bits = 0
        ret = []
        maxv = (1 << tobits) - 1
        max_acc = (1 << (frombits + tobits - 1)) - 1
        for value in data:
            if value < 0 or (value >> frombits):
                return None
            acc = ((acc << frombits) | value) & max_acc
            bits += frombits
            while bits >= tobits:
                bits -= tobits
                ret.append((acc >> bits) & maxv)
        if pad:
            if bits:
                ret.append((acc << (tobits - bits)) & maxv)
        elif bits >= frombits or ((acc << (tobits - bits)) & maxv):
            return None
        return ret
    
    data_8bit = convertbits(data_5bit, 5, 8, False)
    if data_8bit is None:
        return None, None, None
    
    return hrp, witness_version, bytes(data_8bit)

def test_decode_expected_address():
    expected = "bc1p72fvqwm9w4wcsd205maky9qejf6dwa6qeku5f5vnu4phpp3vvpws0p2f4g"
    hrp, version, data = decode_bech32m(expected)
    
    print(f"Expected address: {expected}")
    print(f"HRP: {hrp}")
    print(f"Witness version: {version}")
    print(f"Data: {data.hex() if data else None}")
    print(f"Data length: {len(data) if data else 0}")
    
    # Check what our implementation generates
    from rooch.address.bitcoin import BitcoinAddress
    
    pubkey = bytes.fromhex("034cdb7426f6cebd2e69630c5214fac8dee6a999b43b22907d1d8e4a9363a96a14")
    generated = BitcoinAddress.from_taproot_public_key(pubkey, True)
    
    print(f"\nGenerated address: {generated.address}")
    hrp2, version2, data2 = decode_bech32m(generated.address)
    print(f"HRP: {hrp2}")
    print(f"Witness version: {version2}")
    print(f"Data: {data2.hex() if data2 else None}")
    print(f"Data length: {len(data2) if data2 else 0}")
    
    # Compare the internal keys
    if data and data2:
        print(f"\nExpected internal key: {data.hex()}")
        print(f"Generated internal key: {data2.hex()}")
        
        # Compare with the original x-coordinate from pubkey
        x_coord = pubkey[1:] if len(pubkey) == 33 else pubkey
        print(f"Original x-coordinate: {x_coord.hex()}")

if __name__ == "__main__":
    test_decode_expected_address()
