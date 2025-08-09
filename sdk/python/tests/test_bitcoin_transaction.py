#!/usr/bin/env python3
# Copyright (c) Rooch Network
# SPDX-License-Identifier: Apache-2.0

import hashlib
import pytest
from rooch.crypto.keypair import KeyPair
from rooch.utils.hex import to_hex, from_hex


class TestBitcoinTransaction:
    """Test Bitcoin transaction signing and authentication"""

    @pytest.fixture(autouse=True)
    def setup(self):
        """Set up test fixtures"""
        # Use a fixed private key for reproducible tests
        self.private_key_hex = "0x1234567890abcdef1234567890abcdef1234567890abcdef1234567890abcdef"
        self.keypair = KeyPair.from_private_key(self.private_key_hex)

    def test_bitcoin_authenticator_sign(self):
        """Test Bitcoin authenticator signing process"""
        # Create a test transaction
        tx_hash = b"0123456789abcdef0123456789abcdef"  # 32-byte transaction hash
        
        # Test the exact signing process used in Bitcoin authentication
        message_prefix = "Bitcoin Signed Message:\n"
        message_info = tx_hash.hex()
        
        # Create the sign data exactly as in the authenticator
        sign_data = message_prefix.encode('utf-8') + message_info.encode('utf-8')
        sign_data_hash = hashlib.sha256(sign_data).digest()
        
        # Sign the hash
        signature = self.keypair.sign_digest(sign_data_hash)
        
        # Verify locally
        from ecdsa import util
        vk = self.keypair._ecdsa_verifying_key
        is_valid = vk.verify_digest(signature, sign_data_hash, sigdecode=util.sigdecode_string)
        
        assert is_valid is True
        
        print(f"Bitcoin authenticator test:")
        print(f"  TX hash: {tx_hash.hex()}")
        print(f"  Message prefix: {message_prefix!r}")
        print(f"  Message info: {message_info}")
        print(f"  Sign data: {to_hex(sign_data)}")
        print(f"  Sign data hash: {to_hex(sign_data_hash)}")
        print(f"  Signature: {to_hex(signature)}")
        print(f"  Local verification: {is_valid}")

    def test_bitcoin_auth_payload_creation(self):
        """Test Bitcoin AuthPayload creation"""
        from rooch.bcs.serializer import BcsSerializer, Serializable
        
        # Test data
        tx_hash = b"0123456789abcdef0123456789abcdef"
        message_prefix = "Bitcoin Signed Message:\n"
        message_info = tx_hash.hex()
        
        # Create sign data and hash it
        sign_data = message_prefix.encode('utf-8') + message_info.encode('utf-8')
        sign_data_hash = hashlib.sha256(sign_data).digest()
        
        # Sign the hash
        signature = self.keypair.sign_digest(sign_data_hash)
        
        # Get components for AuthPayload
        public_key = self.keypair.get_public_key()
        from_address = self.keypair.get_rooch_address().to_bytes()
        
        # Create BitcoinAuthPayload structure
        class BitcoinAuthPayload(Serializable):
            def __init__(self, signature: bytes, message_prefix: bytes, message_info: bytes, 
                        public_key: bytes, from_address: bytes):
                self.signature = signature
                self.message_prefix = message_prefix
                self.message_info = message_info
                self.public_key = public_key
                self.from_address = from_address
            
            def serialize(self, serializer: BcsSerializer):
                serializer.bytes(self.signature)
                serializer.bytes(self.message_prefix)
                serializer.bytes(self.message_info)
                serializer.bytes(self.public_key)
                serializer.bytes(self.from_address)
        
        # Create and serialize AuthPayload
        auth_payload = BitcoinAuthPayload(
            signature=signature,
            message_prefix=message_prefix.encode('utf-8'),
            message_info=message_info.encode('utf-8'),
            public_key=public_key,
            from_address=from_address
        )
        
        serializer = BcsSerializer()
        auth_payload.serialize(serializer)
        auth_payload_bytes = serializer.output()
        
        print(f"Bitcoin AuthPayload test:")
        print(f"  Signature: {to_hex(signature)}")
        print(f"  Message prefix: {to_hex(message_prefix.encode('utf-8'))}")
        print(f"  Message info: {to_hex(message_info.encode('utf-8'))}")
        print(f"  Public key: {to_hex(public_key)}")
        print(f"  From address: {to_hex(from_address)}")
        print(f"  AuthPayload serialized: {to_hex(auth_payload_bytes)}")
        
        # Verify the structure is correct
        assert len(signature) == 64
        assert len(public_key) == 65
        assert len(from_address) == 32
        assert len(auth_payload_bytes) > 0

    def test_consensus_codec_encoding(self):
        """Test consensus codec encoding like in Move"""
        # Test the exact encoding used in Move's consensus codec
        test_data = b"Bitcoin Signed Message:\n0123456789abcdef0123456789abcdef"
        
        # Apply varint encoding (consensus codec format)
        def encode_varint(data: bytes) -> bytes:
            """Encode bytes with varint length prefix"""
            data_len = len(data)
            varint_bytes = []
            
            # Encode length as varint
            remaining = data_len
            while remaining >= 0x80:
                varint_bytes.append((remaining & 0x7F) | 0x80)
                remaining >>= 7
            varint_bytes.append(remaining)
            
            return bytes(varint_bytes) + data
        
        encoded_data = encode_varint(test_data)
        encoded_hash = hashlib.sha256(encoded_data).digest()
        
        # Sign the encoded hash
        signature = self.keypair.sign_digest(encoded_hash)
        
        # Verify
        from ecdsa import util
        vk = self.keypair._ecdsa_verifying_key
        is_valid = vk.verify_digest(signature, encoded_hash, sigdecode=util.sigdecode_string)
        
        assert is_valid is True
        
        print(f"Consensus codec test:")
        print(f"  Original data: {to_hex(test_data)}")
        print(f"  Encoded data: {to_hex(encoded_data)}")
        print(f"  Encoded hash: {to_hex(encoded_hash)}")
        print(f"  Signature: {to_hex(signature)}")
        print(f"  Verification: {is_valid}")

    def test_bitcoin_auth_full_flow(self):
        """Test the complete Bitcoin authentication flow"""
        # This test simulates the exact flow from builder.py
        tx_hash = b"0123456789abcdef0123456789abcdef"
        
        # Step 1: Create sign data
        message_prefix = "Bitcoin Signed Message:\n"
        message_info = tx_hash.hex()
        sign_data = message_prefix.encode('utf-8') + message_info.encode('utf-8')
        
        # Step 2: Apply consensus codec encoding
        def encode_varint(data: bytes) -> bytes:
            data_len = len(data)
            if data_len < 0x80:
                return bytes([data_len]) + data
            else:
                varint_bytes = []
                remaining = data_len
                while remaining >= 0x80:
                    varint_bytes.append((remaining & 0x7F) | 0x80)
                    remaining >>= 7
                varint_bytes.append(remaining)
                return bytes(varint_bytes) + data
        
        sign_data_encoded = encode_varint(sign_data)
        
        # Step 3: Hash the encoded data
        sign_data_hash = hashlib.sha256(sign_data_encoded).digest()
        
        # Step 4: Sign the hash
        signature = self.keypair.sign_digest(sign_data_hash)
        
        # Step 5: Create AuthPayload
        public_key = self.keypair.get_public_key()
        from_address = self.keypair.get_rooch_address().to_bytes()
        
        # Step 6: Verify signature locally
        from ecdsa import util
        vk = self.keypair._ecdsa_verifying_key
        is_valid = vk.verify_digest(signature, sign_data_hash, sigdecode=util.sigdecode_string)
        
        assert is_valid is True
        
        print(f"Bitcoin auth full flow test:")
        print(f"  TX hash: {tx_hash.hex()}")
        print(f"  Sign data: {to_hex(sign_data)}")
        print(f"  Sign data encoded: {to_hex(sign_data_encoded)}")
        print(f"  Sign data hash: {to_hex(sign_data_hash)}")
        print(f"  Signature: {to_hex(signature)}")
        print(f"  Public key: {to_hex(public_key)}")
        print(f"  From address: {to_hex(from_address)}")
        print(f"  Local verification: {is_valid}")
        
        # All components should be valid
        assert len(signature) == 64
        assert len(public_key) == 65
        assert len(from_address) == 32
        assert len(sign_data_hash) == 32


if __name__ == '__main__':
    import sys
    import pytest
    sys.exit(pytest.main([__file__] + sys.argv[1:]))
