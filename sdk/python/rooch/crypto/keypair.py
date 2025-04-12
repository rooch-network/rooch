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

from ecdsa import SigningKey, VerifyingKey, NIST256p, util

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

        # Create corresponding ecdsa keys using the private scalar 'd'
        try:
            # Ensure the key has the private component 'd'
            if not hasattr(self._key, 'd'):
                 raise ValueError("Cannot create ecdsa key from public key")
                 
            # Use from_secret_exponent with the integer d
            self._ecdsa_signing_key = SigningKey.from_secret_exponent(self._key.d, curve=NIST256p)
            self._ecdsa_verifying_key = self._ecdsa_signing_key.verifying_key
        except ValueError as ve:
             # Re-raise specific ValueError if needed
             raise ve
        except Exception as e:
            # Catch potential errors during ecdsa key creation
            raise RuntimeError(f"Failed to create ecdsa key from secret exponent: {e}") from e
    
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
            instance._key = ecc_key      # Set the pycryptodome key directly
            
            # Manually add the ecdsa key initialization here, as __init__ is skipped
            try:
                # Use the derived scalar 'd' directly to create the ecdsa key
                instance._ecdsa_signing_key = SigningKey.from_secret_exponent(d, curve=NIST256p)
                instance._ecdsa_verifying_key = instance._ecdsa_signing_key.verifying_key
            except Exception as e_ecdsa:
                raise RuntimeError(f"Failed to create ecdsa key within from_seed using secret exponent: {e_ecdsa}") from e_ecdsa
                
            return instance
        except ValueError as e_ecc:
            # Handle cases where d might be invalid for the curve (e.g., 0 or >= curve order)
            raise ValueError(f"Failed to construct ECC key from seed: {e_ecc}") from e_ecc
    
    @classmethod
    def generate(cls) -> 'KeyPair':
        """Generate a new random key pair
        
        Returns:
            KeyPair instance
        """
        return cls()
    
    def get_public_key(self) -> bytes:
        """Get the public key in uncompressed format (65 bytes: 0x04 + X + Y)
        using the ecdsa library.
        
        Returns:
            Public key as bytes (uncompressed format)
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
        """Sign a pre-computed digest (hash) using the ecdsa library.

        Args:
            digest: The 32-byte digest to sign.

        Returns:
            Signature as raw bytes (R || S, 64 bytes for P-256).

        Raises:
            ValueError: If digest length is not 32 bytes.
        """
        if len(digest) != 32:
             raise ValueError(f"Digest must be 32 bytes long, got {len(digest)}")
        
        # Use ecdsa SigningKey to sign the raw digest.
        # sigencode_string ensures the output is raw R || S bytes.
        try:
            signature = self._ecdsa_signing_key.sign_digest(
                digest,
                sigencode=util.sigencode_string
            )
            # NIST256p (P-256) uses 32-byte R and 32-byte S
            assert len(signature) == 64, "Signature should be 64 bytes (R || S)"
            return signature
        except Exception as e:
             # Catch potential errors during signing
             raise RuntimeError(f"Failed to sign digest using ecdsa: {e}") from e

    def sign(self, message: Union[str, bytes]) -> bytes:
        """Sign a message (hashes with SHA256 first). Returns DER encoded signature.
           NOTE: For Rooch transactions, use sign_digest with SHA3-256 hash.
        Args:
            message: Message to sign
            
        Returns:
            Signature as DER encoded bytes
        """
        if isinstance(message, str):
            if message.startswith('0x'):
                message = from_hex(message)
            else:
                message = message.encode('utf-8')
        
        # This method inherently uses SHA256 due to pycryptodome DSS limitations
        h = SHA256.new(message)
        try:
            from Crypto.Signature import DSS # Local import if keeping method
            signer = DSS.new(self._key, 'fips-186-3')
            return signer.sign(h)
        except ImportError:
            raise RuntimeError("pycryptodome is required for the legacy sign() method.")
    
    def sign_hex(self, message: Union[str, bytes]) -> str:
        """Sign a message and return the signature as hex
        
        Args:
            message: Message to sign
            
        Returns:
            Signature as hex string
        """
        return to_hex(self.sign(message))
    
    def verify(self, message: Union[str, bytes], signature: Union[str, bytes]) -> bool:
        """Verify a signature against the original message using ecdsa.
           NOTE: This uses SHA256 for hashing, matching the legacy sign() method.

        Args:
            message: Original message
            signature: Signature to verify (DER encoded or raw R||S bytes)

        Returns:
            True if the signature is valid
        """
        if isinstance(message, str):
            if message.startswith('0x'):
                message = from_hex(message)
            else:
                message = message.encode('utf-8')

        # Hash the message using SHA256 (to match legacy sign method)
        h = SHA256.new(message)
        digest = h.digest()

        if isinstance(signature, str):
            signature_bytes = from_hex(ensure_hex_prefix(signature))
        else:
            signature_bytes = signature

        try:
            # Try verifying assuming raw R||S format first
            if len(signature_bytes) == 64:
                return self._ecdsa_verifying_key.verify_digest(
                    signature_bytes,
                    digest,
                    sigdecode=util.sigdecode_string
                )
            else:
                # Assume DER format if not raw bytes
                return self._ecdsa_verifying_key.verify_digest(
                    signature_bytes,
                    digest,
                    sigdecode=util.sigdecode_der
                )
        except (ImportError, ecdsa.BadSignatureError, ValueError, TypeError, IndexError):
             # Catch errors from ecdsa or hex decoding
             return False
    
    def get_rooch_address(self) -> RoochAddress:
        """Get the Rooch address associated with this key pair.
        Note: Uses SHA256 hash of the uncompressed public key.
        """
        # Ensure we use the updated get_public_key() which uses ecdsa
        public_key = self.get_public_key()
        # Rooch address generation seems to use SHA256 of the public key bytes
        address_bytes = hashlib.sha256(public_key).digest()
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