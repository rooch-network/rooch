#!/usr/bin/env python3
# Copyright (c) Rooch Network
# SPDX-License-Identifier: Apache-2.0

import hashlib
import hmac
import os
from typing import Dict, Optional, Tuple, Union

from Crypto.PublicKey import ECC
from Crypto.Hash import SHA256
from Crypto.Util.number import bytes_to_long, long_to_bytes

from ecdsa import SigningKey, VerifyingKey, SECP256k1, util

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
            # Generate a new key pair using SECP256k1
            # Note: pycryptodome does not directly support generating SECP256k1 keys.
            # We will generate using ecdsa first, then potentially import into pycryptodome if needed.
            temp_ecdsa_sk = SigningKey.generate(curve=SECP256k1)
            private_key_bytes = temp_ecdsa_sk.to_string()
            # Store the ecdsa keys directly
            self._ecdsa_signing_key = temp_ecdsa_sk
            self._ecdsa_verifying_key = temp_ecdsa_sk.verifying_key
            # We may not need self._key (pycryptodome) if ecdsa handles all?
            # If needed for compatibility, try importing:
            # self._key = ECC.construct(curve='secp256k1', d=temp_ecdsa_sk.privkey.secret_multiplier)
            # For now, assume ecdsa is sufficient and remove dependency on self._key for secp256k1
            self._key = None # Or handle potential import error
        else:
            # Use provided private key (assume it's secp256k1)
            try:
                if isinstance(private_key, str):
                    private_key_bytes = from_hex(ensure_hex_prefix(private_key))
                else:
                    private_key_bytes = private_key
                    
                if len(private_key_bytes) != 32:
                    raise ValueError("Secp256k1 private key must be 32 bytes")

                # Create ecdsa keys from raw private key bytes
                self._ecdsa_signing_key = SigningKey.from_string(private_key_bytes, curve=SECP256k1)
                self._ecdsa_verifying_key = self._ecdsa_signing_key.verifying_key
                self._key = None # Assuming ecdsa is sufficient

            except Exception as e:
                raise ValueError(f"Invalid secp256k1 private key: {str(e)}")

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
        d_bytes = hmac.new(b'ROOCH_KEYPAIR_SEED', seed, hashlib.sha256).digest()
        # d = bytes_to_long(key_bytes) # Not needed directly

        # Construct the keys directly from the derived bytes
        try:
            # Create an instance without calling __init__
            instance = cls.__new__(cls)
            
            # Initialize ecdsa keys directly from the seed-derived bytes
            instance._ecdsa_signing_key = SigningKey.from_string(d_bytes, curve=SECP256k1)
            instance._ecdsa_verifying_key = instance._ecdsa_signing_key.verifying_key
            instance._key = None # Mark pycryptodome key as unused/unavailable
                
            return instance
        except Exception as e:
            raise ValueError(f"Failed to construct secp256k1 key from seed: {e}") from e
    
    @classmethod
    def generate(cls) -> 'KeyPair':
        """Generate a new random key pair
        
        Returns:
            KeyPair instance
        """
        return cls()
    
    def get_public_key(self) -> bytes:
        """Get the public key in uncompressed format (65 bytes: 0x04 + X + Y)
           For SECP256k1.
        """
        # Use ecdsa's verifying_key to get the uncompressed format
        return self._ecdsa_verifying_key.to_string('uncompressed')
    
    def get_public_key_hex(self) -> str:
        """Get the public key as hex
        
        Returns:
            Public key as hex string
        """
        return to_hex(self.get_public_key())
    
    def get_private_key(self) -> bytes:
        """Get the private key bytes (32 bytes for secp256k1)."""
        return self._ecdsa_signing_key.to_string()
    
    def get_private_key_hex(self) -> str:
        """Get the private key as hex."""
        return to_hex(self.get_private_key())
    
    def sign_digest(self, digest: bytes) -> bytes:
        """Sign a pre-computed digest (hash) using the ecdsa library (SECP256k1).
           Returns raw R || S format (64 bytes).
        """
        if len(digest) != 32:
             raise ValueError(f"Digest must be 32 bytes long, got {len(digest)}")
        
        try:
            # SECP256k1 uses recoverable signatures by default, but sign_digest is non-recoverable.
            # Ensure we get the standard R||S format.
            signature = self._ecdsa_signing_key.sign_digest(
                digest,
                sigencode=util.sigencode_string
            )
            assert len(signature) == 64
            return signature
        except Exception as e:
             raise RuntimeError(f"Failed to sign digest using ecdsa (secp256k1): {e}") from e

    def sign(self, message: Union[str, bytes]) -> bytes:
        raise NotImplementedError("Legacy sign() method not compatible with SECP256k1 setup. Use sign_digest.")
    
    def sign_hex(self, message: Union[str, bytes]) -> str:
        """Sign a message and return the signature as hex
        
        Args:
            message: Message to sign
            
        Returns:
            Signature as hex string
        """
        return to_hex(self.sign(message))
    
    def verify(self, message: Union[str, bytes], signature: Union[str, bytes]) -> bool:
        raise NotImplementedError("Verify method needs update for secp256k1 and SHA3 hash.")
    
    def get_rooch_address(self) -> RoochAddress:
        """Get the Rooch address. ASSUMPTION: Uses SHA256 hash of uncompressed pubkey."""
        public_key = self.get_public_key()
        address_bytes = hashlib.sha256(public_key).digest()
        assert len(address_bytes) == 32
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