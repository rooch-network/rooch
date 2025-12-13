#!/usr/bin/env python3
# Copyright (c) Rooch Network
# SPDX-License-Identifier: Apache-2.0

import hashlib
import pytest
from rooch.crypto.keypair import KeyPair
from rooch.utils.hex import to_hex, from_hex


class TestCrypto:
    """Test cryptographic operations"""

    @pytest.fixture(autouse=True)
    def setup(self):
        """Set up test fixtures"""
        # Use a fixed private key for reproducible tests
        self.private_key_hex = "0x1234567890abcdef1234567890abcdef1234567890abcdef1234567890abcdef"
        self.keypair = KeyPair.from_private_key(self.private_key_hex)

    def test_keypair_generation(self):
        """Test key pair generation"""
        # Test generation from private key
        kp = KeyPair.from_private_key(self.private_key_hex)
        assert kp.get_private_key_hex() == self.private_key_hex
        
        # Test random generation
        kp_random = KeyPair.generate()
        assert kp_random.get_private_key() is not None
        assert len(kp_random.get_private_key()) == 32
        
        # Test seed generation
        seed = "test_seed_12345"
        kp_seed1 = KeyPair.from_seed(seed)
        kp_seed2 = KeyPair.from_seed(seed)
        # Same seed should produce same key
        assert kp_seed1.get_private_key_hex() == kp_seed2.get_private_key_hex()

    def test_public_key_format(self):
        """Test public key format"""
        public_key = self.keypair.get_public_key()
        # Should be 65 bytes (uncompressed format: 0x04 + X + Y)
        assert len(public_key) == 65
        assert public_key[0] == 0x04  # Uncompressed format marker

    def test_sign_and_verify_message(self):
        """Test signing and verifying a message"""
        message = "Hello, Rooch!"
        
        # Sign the message
        signature = self.keypair.sign(message)
        assert len(signature) == 64  # R || S format
        
        # Verify the signature
        is_valid = self.keypair.verify(message, signature)
        assert is_valid is True
        
        # Test with different message
        is_valid_wrong = self.keypair.verify("Wrong message", signature)
        assert is_valid_wrong is False

    def test_sign_digest_deterministic(self):
        """Test deterministic digest signing"""
        # Create a test digest (32 bytes)
        test_digest = hashlib.sha256(b"test_message").digest()
        
        # Sign the same digest multiple times
        sig1 = self.keypair.sign_digest(test_digest)
        sig2 = self.keypair.sign_digest(test_digest)
        
        # Should be deterministic (same signature each time)
        assert sig1 == sig2
        assert len(sig1) == 64  # R || S format
        
        print(f"Test digest: {to_hex(test_digest)}")
        print(f"Signature: {to_hex(sig1)}")

    def test_digest_verification(self):
        """Test digest verification directly"""
        # Create a test digest
        test_message = b"test_bitcoin_auth_message"
        test_digest = hashlib.sha256(test_message).digest()
        
        # Sign the digest
        signature = self.keypair.sign_digest(test_digest)
        
        # Verify using digest directly
        from ecdsa import util
        vk = self.keypair._ecdsa_verifying_key
        
        try:
            # This should succeed
            is_valid = vk.verify_digest(signature, test_digest, sigdecode=util.sigdecode_string)
            assert is_valid is True
            print("✓ Digest verification successful")
        except Exception as e:
            pytest.fail(f"Digest verification failed: {e}")
        
        # Test with wrong digest
        wrong_digest = hashlib.sha256(b"wrong_message").digest()
        try:
            is_valid_wrong = vk.verify_digest(signature, wrong_digest, sigdecode=util.sigdecode_string)
            assert is_valid_wrong is False
        except Exception:
            # Verification should fail gracefully
            pass

    def test_bitcoin_auth_signature_format(self):
        """Test signature format for Bitcoin authentication"""
        # Simulate the exact process from Bitcoin authentication
        message_prefix = "Bitcoin Signed Message:\n"
        message_info = "test_tx_hash"
        public_key = self.keypair.get_public_key()
        from_address = self.keypair.get_rooch_address().to_bytes()
        tx_hash = b"test_transaction_hash_32_bytes_"
        
        # Create sign_data like in Bitcoin auth
        sign_data = message_prefix.encode('utf-8') + message_info.encode('utf-8')
        sign_data_hash = hashlib.sha256(sign_data).digest()
        
        # Sign the hash
        signature = self.keypair.sign_digest(sign_data_hash)
        
        # Verify the signature
        vk = self.keypair._ecdsa_verifying_key
        from ecdsa import util
        
        try:
            is_valid = vk.verify_digest(signature, sign_data_hash, sigdecode=util.sigdecode_string)
            assert is_valid is True
            
            print(f"Bitcoin auth test:")
            print(f"  Sign data: {to_hex(sign_data)}")
            print(f"  Sign data hash: {to_hex(sign_data_hash)}")
            print(f"  Signature: {to_hex(signature)}")
            print(f"  Verification: {is_valid}")
            
        except Exception as e:
            pytest.fail(f"Bitcoin auth signature verification failed: {e}")

    def test_cross_keypair_verification(self):
        """Test that one keypair cannot verify another's signature"""
        message = "test_message"
        
        # Create two different keypairs
        kp1 = KeyPair.generate()
        kp2 = KeyPair.generate()
        
        # Sign with kp1
        signature = kp1.sign(message)
        
        # Verify with kp1 (should succeed)
        assert kp1.verify(message, signature) is True
        
        # Verify with kp2 (should fail)
        assert kp2.verify(message, signature) is False

    def test_bitcoin_authenticator_sign(self):
        """Test Bitcoin authenticator signing process"""
        # Test the exact signing process used in Bitcoin authentication
        tx_hash = b"0" * 32  # 32-byte transaction hash
        message_prefix = "Bitcoin Signed Message:\n"
        message_info = tx_hash.hex()
        
        # Create the sign data exactly as in the authenticator
        sign_data = message_prefix.encode('utf-8') + message_info.encode('utf-8')
        
        # Single SHA256 (for comparison)
        single_hash = hashlib.sha256(sign_data).digest()
        
        # Double SHA256 (to match Rust verify_with_hash::<Sha256>)
        first_hash = hashlib.sha256(sign_data).digest()
        double_hash = hashlib.sha256(first_hash).digest()
        
        # Test both methods
        single_signature = self.keypair.sign_digest(single_hash)
        double_signature = self.keypair.sign_digest(double_hash)
        
        # Verify locally with single hash
        from ecdsa import util
        vk = self.keypair._ecdsa_verifying_key
        single_valid = vk.verify_digest(single_signature, single_hash, sigdecode=util.sigdecode_string)
        
        # Verify locally with double hash
        double_valid = vk.verify_digest(double_signature, double_hash, sigdecode=util.sigdecode_string)
        
        assert single_valid is True
        assert double_valid is True
        
        print(f"Bitcoin authenticator test:")
        print(f"  TX hash: {tx_hash.hex()}")
        print(f"  Message prefix: {message_prefix!r}")
        print(f"  Message info: {message_info}")
        print(f"  Sign data: {to_hex(sign_data)}")
        print(f"  Single SHA256: {to_hex(single_hash)}")
        print(f"  Double SHA256: {to_hex(double_hash)}")
        print(f"  Single signature: {to_hex(single_signature)}")
        print(f"  Double signature: {to_hex(double_signature)}")
        print(f"  Single verification: {single_valid}")
        print(f"  Double verification: {double_valid}")

    def test_consensus_codec_simulation(self):
        """Test consensus codec encoding simulation"""
        # Simulate the consensus codec format used in Move
        test_data = b"Hello, World!"
        
        # Simulate varint encoding (simple case for small data)
        data_len = len(test_data)
        if data_len < 0x80:
            varint_encoded = bytes([data_len]) + test_data
        else:
            # More complex varint encoding for larger data
            varint_bytes = []
            remaining = data_len
            while remaining >= 0x80:
                varint_bytes.append((remaining & 0x7F) | 0x80)
                remaining >>= 7
            varint_bytes.append(remaining)
            varint_encoded = bytes(varint_bytes) + test_data
        
        print(f"Original data: {to_hex(test_data)}")
        print(f"Varint encoded: {to_hex(varint_encoded)}")
        
        # Test with actual message like in Bitcoin auth
        message_prefix = "Bitcoin Signed Message:\n"
        message_info = "test_tx_hash"
        sign_data = message_prefix.encode('utf-8') + message_info.encode('utf-8')
        
        # Apply varint encoding
        data_len = len(sign_data)
        if data_len < 0x80:
            encoded_sign_data = bytes([data_len]) + sign_data
        else:
            varint_bytes = []
            remaining = data_len
            while remaining >= 0x80:
                varint_bytes.append((remaining & 0x7F) | 0x80)
                remaining >>= 7
            varint_bytes.append(remaining)
            encoded_sign_data = bytes(varint_bytes) + sign_data
        
        # Hash the encoded data
        encoded_hash = hashlib.sha256(encoded_sign_data).digest()
        
        print(f"Sign data: {to_hex(sign_data)}")
        print(f"Encoded sign data: {to_hex(encoded_sign_data)}")
        print(f"Encoded hash: {to_hex(encoded_hash)}")
        
        # Sign and verify
        signature = self.keypair.sign_digest(encoded_hash)
        from ecdsa import util
        vk = self.keypair._ecdsa_verifying_key
        is_valid = vk.verify_digest(signature, encoded_hash, sigdecode=util.sigdecode_string)
        
        assert is_valid is True
        print(f"Signature verification: {is_valid}")


if __name__ == '__main__':
    import sys
    import pytest
    sys.exit(pytest.main([__file__] + sys.argv[1:]))
