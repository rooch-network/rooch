#!/usr/bin/env python3
"""Test signature compatibility with Move ecdsa_k1::verify"""

import hashlib
from rooch.crypto.keypair import KeyPair
from rooch.utils.hex import to_hex, from_hex

def test_move_compatibility():
    """Test if our signatures match Move's expectations"""
    
    # From Move test: 
    # let msg = x"00010203";
    # let pubkey = x"033e99a541db69bd32040dfe5037fbf5210dafa8151a71e21c5204b05d95ce0a62";
    # let sig = x"416a21d50b3c838328d4f03213f8ef0c3776389a972ba1ecd37b56243734eba208ea6aaa6fc076ad7accd71d355f693a6fe54fe69b3c168eace9803827bc9046";
    
    move_msg = from_hex("00010203")
    move_pubkey = from_hex("033e99a541db69bd32040dfe5037fbf5210dafa8151a71e21c5204b05d95ce0a62")
    move_signature = from_hex("416a21d50b3c838328d4f03213f8ef0c3776389a972ba1ecd37b56243734eba208ea6aaa6fc076ad7accd71d355f693a6fe54fe69b3c168eace9803827bc9046")
    
    print(f"Move test case:")
    print(f"  Message: {to_hex(move_msg)}")
    print(f"  Public key: {to_hex(move_pubkey)}")
    print(f"  Signature: {to_hex(move_signature)}")
    print(f"  Message length: {len(move_msg)}")
    print(f"  Public key length: {len(move_pubkey)}")
    print(f"  Signature length: {len(move_signature)}")
    
    # Now try to reproduce this with our KeyPair
    # We need to find the private key that corresponds to the public key
    # For testing, let's create a known key and compare formats
    
    test_private_key = "0x1234567890abcdef1234567890abcdef1234567890abcdef1234567890abcdef"
    kp = KeyPair.from_private_key(test_private_key)
    
    # Get compressed public key
    uncompressed_pubkey = kp.get_public_key()
    compressed_pubkey = kp._ecdsa_verifying_key.to_string('compressed')
    
    print(f"\nOur KeyPair:")
    print(f"  Private key: {test_private_key}")
    print(f"  Uncompressed public key: {to_hex(uncompressed_pubkey)} ({len(uncompressed_pubkey)} bytes)")
    print(f"  Compressed public key: {to_hex(compressed_pubkey)} ({len(compressed_pubkey)} bytes)")
    
    # Test signing the Move test message
    test_message = move_msg
    
    # Method 1: Sign digest directly (what Move expects)
    message_hash = hashlib.sha256(test_message).digest()
    signature1 = kp.sign_digest(message_hash)
    
    print(f"\nSigning test message {to_hex(test_message)}:")
    print(f"  Message SHA256: {to_hex(message_hash)}")
    print(f"  Our signature: {to_hex(signature1)} ({len(signature1)} bytes)")
    
    # Verify locally
    from ecdsa import util
    vk = kp._ecdsa_verifying_key
    try:
        is_valid = vk.verify_digest(signature1, message_hash, sigdecode=util.sigdecode_string)
        print(f"  Local verification: {is_valid}")
    except Exception as e:
        print(f"  Local verification failed: {e}")
    
    # Test different signature formats
    print(f"\nTesting different signature methods:")
    
    # Method 2: Raw data signing
    signature2 = kp.sign_raw_data(test_message)
    print(f"  Raw data signature: {to_hex(signature2)}")
    
    # Method 3: Regular sign (SHA3)
    signature3 = kp.sign(test_message)
    print(f"  SHA3 signature: {to_hex(signature3)}")
    
    print(f"\nFormat comparison:")
    print(f"  Move signature format: {len(move_signature)} bytes")
    print(f"  Our signature format: {len(signature1)} bytes")
    print(f"  Formats match: {len(move_signature) == len(signature1)}")

if __name__ == "__main__":
    test_move_compatibility()
