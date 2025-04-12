#!/usr/bin/env python3
# Copyright (c) Rooch Network
# SPDX-License-Identifier: Apache-2.0

import hashlib
import hmac
import os
from typing import Dict, Optional, Tuple, Union

from Crypto.PublicKey import ECC
from Crypto.Hash import SHA256
from Crypto.Signature import DSS
from Crypto.Util.number import bytes_to_long

from ..address.rooch import RoochAddress
from ..utils.hex import ensure_hex_prefix, from_hex, to_hex


class KeyPair:
    """Key pair for signing transactions"""
    
    def __init__(self, private_key: Optional[Union[str, bytes]] = None):
        """Initialize a key pair
        
        Args:
            private_key: Optional private key as hex string or bytes. If not provided,
                        a new key pair will be generated
        
        Raises:
            ValueError: If the private key is invalid
        """
        if private_key is None:
            # Generate a new key pair
            self._key = ECC.generate(curve='P-256')
        else:
            # Use provided private key
            try:
                if isinstance(private_key, str):
                    # Convert hex string to bytes
                    private_key = from_hex(ensure_hex_prefix(private_key))
                
                self._key = ECC.import_key(private_key)
            except (ValueError, TypeError) as e:
                raise ValueError(f"Invalid private key: {str(e)}")
    
    @classmethod
    def from_private_key(cls, private_key: Union[str, bytes]) -> 'KeyPair':
        """Create a key pair from a private key
        
        Args:
            private_key: Private key as hex string or bytes
            
        Returns:
            KeyPair instance
        """
        return cls(private_key)
    
    @classmethod
    def from_seed(cls, seed: Union[str, bytes]) -> 'KeyPair':
        """Create a key pair from a seed
        
        Args:
            seed: Seed as string or bytes
            
        Returns:
            KeyPair instance
        """
        if isinstance(seed, str):
            seed = seed.encode('utf-8')
        
        # Derive private key scalar 'd' from seed using HMAC-SHA256
        key_bytes = hmac.new(b'ROOCH_KEYPAIR_SEED', seed, hashlib.sha256).digest()
        d = bytes_to_long(key_bytes)

        # Construct the ECC key directly from the scalar 'd'
        try:
            ecc_key = ECC.construct(curve='P-256', d=d)
            # Create an instance without calling the default __init__ logic for private_key
            instance = cls.__new__(cls) # Create instance without calling __init__
            instance._key = ecc_key      # Set the key directly
            return instance
        except ValueError as e:
            # Handle cases where d might be invalid for the curve (e.g., 0 or >= curve order)
            raise ValueError(f"Failed to construct key from seed: {e}")
    
    @classmethod
    def generate(cls) -> 'KeyPair':
        """Generate a new random key pair
        
        Returns:
            KeyPair instance
        """
        return cls()
    
    def get_public_key(self) -> bytes:
        """Get the public key in uncompressed format (65 bytes: 0x04 + X + Y)
        
        Returns:
            Public key as bytes (uncompressed format)
        """
        # export_key(format='raw') for P-256 gives 64 bytes (X || Y)
        raw_key = self._key.public_key().export_key(format='raw')
        # Prepend the 0x04 byte to indicate uncompressed format
        return b'\x04' + raw_key
    
    def get_public_key_hex(self) -> str:
        """Get the public key as hex
        
        Returns:
            Public key as hex string
        """
        return to_hex(self.get_public_key())
    
    def get_private_key(self) -> bytes:
        """Get the private key
        
        Returns:
            Private key as bytes
        """
        return self._key.export_key(format='raw')
    
    def get_private_key_hex(self) -> str:
        """Get the private key as hex
        
        Returns:
            Private key as hex string
        """
        return to_hex(self.get_private_key())
    
    def sign_digest(self, digest: bytes) -> bytes:
        """Sign a pre-computed digest (hash).

        Args:
            digest: The 32-byte digest to sign.

        Returns:
            Signature as raw bytes (R || S).

        Raises:
            ValueError: If digest length is not 32 bytes.
            ImportError: If pycryptodome is not installed correctly.
        """
        if len(digest) != 32:
             # Or perhaps hash length of the curve? P-256 uses SHA-256 (32 bytes)
             # SHA3-256 is also 32 bytes. So this check should be fine.
             raise ValueError(f"Digest must be 32 bytes long, got {len(digest)}")
        
        # Need to wrap the raw digest in a hash object structure for DSS
        # Since the hash is already computed, we create a dummy hash object
        # Note: This is a slight workaround. Ideally, DSS would take raw digest.
        class DummyHash:
            def __init__(self, data):
                self.digest_size = len(data)
                self._digest = data
                # Pretend to be SHA256 to satisfy DSS check
                self.name = 'sha256' 
            def update(self, data): # pragma: no cover
                pass # No-op
            def digest(self): # pragma: no cover
                return self._digest
            def new(self, data=None): # pragma: no cover
                # Required by some internal checks? Return a new instance if needed.
                return DummyHash(data if data is not None else self._digest)

        dummy_hash_obj = DummyHash(digest)
        
        try:
            signer = DSS.new(self._key, 'fips-186-3')
            signature = signer.sign(dummy_hash_obj) # Pass the dummy hash object
            # DSS.sign returns DER encoded signature. We need raw R || S (64 bytes for P-256)
            # We need to decode DER to get R and S.
            from Crypto.Util.asn1 import DerSequence
            from Crypto.Util.number import bytes_to_long, long_to_bytes

            der_seq = DerSequence()
            der_seq.decode(signature)
            r = der_seq[0]
            s = der_seq[1]

            # Convert R and S to fixed 32-byte big-endian representation
            # P-256 curve order is 256 bits (32 bytes)
            n_bytes = 32 # Curve order / 8
            r_bytes = long_to_bytes(r, n_bytes)
            s_bytes = long_to_bytes(s, n_bytes)

            return r_bytes + s_bytes
        except (ImportError, ValueError, TypeError, IndexError) as e:
            # Catch potential errors during signing or DER decoding
            raise RuntimeError(f"Failed to sign digest or decode signature: {e}") from e

    def sign(self, message: Union[str, bytes]) -> bytes:
        """Sign a message
        
        Args:
            message: Message to sign
            
        Returns:
            Signature as bytes
        """
        if isinstance(message, str):
            if message.startswith('0x'):
                message = from_hex(message)
            else:
                message = message.encode('utf-8')
        
        h = SHA256.new(message)
        signer = DSS.new(self._key, 'fips-186-3')
        return signer.sign(h)
    
    def sign_hex(self, message: Union[str, bytes]) -> str:
        """Sign a message and return the signature as hex
        
        Args:
            message: Message to sign
            
        Returns:
            Signature as hex string
        """
        return to_hex(self.sign(message))
    
    def verify(self, message: Union[str, bytes], signature: Union[str, bytes]) -> bool:
        """Verify a signature
        
        Args:
            message: Original message
            signature: Signature to verify
            
        Returns:
            True if the signature is valid
        """
        if isinstance(message, str):
            if message.startswith('0x'):
                message = from_hex(message)
            else:
                message = message.encode('utf-8')
        
        if isinstance(signature, str):
            signature = from_hex(ensure_hex_prefix(signature))
        
        h = SHA256.new(message)
        verifier = DSS.new(self._key.public_key(), 'fips-186-3')
        
        try:
            verifier.verify(h, signature)
            return True
        except ValueError:
            return False
    
    def get_rooch_address(self) -> RoochAddress:
        """Get the Rooch address associated with this key pair
        
        Returns:
            RoochAddress instance (32 bytes from SHA256 hash of public key)
        """
        public_key = self.get_public_key()
        # Hash the public key using SHA256 and use the full 32 bytes for the address
        address_bytes = hashlib.sha256(public_key).digest()
        # Ensure address_bytes is 32 bytes long (SHA256 produces 32 bytes)
        assert len(address_bytes) == 32, "SHA256 hash should be 32 bytes"
        return RoochAddress(address_bytes)
    
    def to_dict(self) -> Dict[str, str]:
        """Convert the key pair to a dictionary
        
        Returns:
            Dictionary containing the key pair information
        """
        return {
            'private_key': self.get_private_key_hex(),
            'public_key': self.get_public_key_hex(),
            'address': str(self.get_rooch_address())
        }