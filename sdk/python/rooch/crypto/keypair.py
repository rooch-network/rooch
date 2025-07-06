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
           Uses deterministic signatures with canonical (low-S) form for Bitcoin compatibility.
        """
        if len(digest) != 32:
             raise ValueError(f"Digest must be 32 bytes long, got {len(digest)}")
        
        try:
            # Use deterministic signatures for Bitcoin compatibility and consistency
            signature = self._ecdsa_signing_key.sign_digest_deterministic(
                digest,
                sigencode=util.sigencode_string
            )
            assert len(signature) == 64
            
            # Ensure canonical signature (low-S value) for Bitcoin compatibility
            # Extract R and S values (each 32 bytes)
            r_bytes = signature[:32]
            s_bytes = signature[32:]
            
            # Convert S to integer
            s = int.from_bytes(s_bytes, byteorder='big')
            
            # SECP256k1 order (n)
            n = 0xFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFEBAAEDCE6AF48A03BBFD25E8CD0364141
            
            # If S > n/2, use n - S (canonical form)
            if s > n // 2:
                s = n - s
                s_bytes = s.to_bytes(32, byteorder='big')
                signature = r_bytes + s_bytes
            
            return signature
        except Exception as e:
            raise RuntimeError(f"Failed to sign digest using ecdsa (secp256k1): {e}") from e

    def sign_raw_data(self, data: bytes) -> bytes:
        """Sign raw data directly using SHA256 hash, compatible with Move's ecdsa_k1::verify.
           This method uses SHA256 (not SHA3) and returns raw R || S format (64 bytes).
           Uses deterministic signatures with canonical (low-S) format for Bitcoin compatibility.
        """
        try:
            # Hash the data with SHA256 (compatible with Move's ecdsa_k1::verify)
            digest = hashlib.sha256(data).digest()
            
            # Always use deterministic signatures for consistency with Rust/Move
            signature = self._ecdsa_signing_key.sign_digest_deterministic(
                digest,
                sigencode=util.sigencode_string
            )
            assert len(signature) == 64
            
            # Ensure canonical signature (low-S value) for Bitcoin compatibility
            # Extract R and S values (each 32 bytes)
            r_bytes = signature[:32]
            s_bytes = signature[32:]
            
            # Convert S to integer
            s = int.from_bytes(s_bytes, byteorder='big')
            
            # SECP256k1 order (n)
            n = 0xFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFEBAAEDCE6AF48A03BBFD25E8CD0364141
            
            # If S > n/2, use n - S (canonical form)
            if s > n // 2:
                s = n - s
                s_bytes = s.to_bytes(32, byteorder='big')
                signature = r_bytes + s_bytes
            
            return signature
        except Exception as e:
            raise RuntimeError(f"Failed to sign raw data using ecdsa (secp256k1): {e}") from e

    def sign(self, message: Union[str, bytes]) -> bytes:
        """Sign a message by hashing it with SHA3-256 and then signing the digest.
        Returns raw R || S format (64 bytes).
        """
        if isinstance(message, str):
            message = message.encode('utf-8')
        digest = hashlib.sha3_256(message).digest()
        return self.sign_digest(digest)
    
    def sign_hex(self, message: Union[str, bytes]) -> str:
        """Sign a message and return the signature as hex
        
        Args:
            message: Message to sign
            
        Returns:
            Signature as hex string
        """
        return to_hex(self.sign(message))
    
    def verify(self, message: Union[str, bytes], signature: Union[str, bytes]) -> bool:
        """Verify a message signature.
        
        Args:
            message: The original message (str or bytes).
            signature: The signature to verify (str or bytes, hex or raw bytes).
            
        Returns:
            True if the signature is valid, False otherwise.
        """
        if isinstance(message, str):
            message = message.encode('utf-8')
        if isinstance(signature, str):
            signature = from_hex(signature)

        digest = hashlib.sha3_256(message).digest()
        
        try:
            # Verify the signature using the public key and the digest
            return self._ecdsa_verifying_key.verify_digest(signature, digest, sigdecode=util.sigdecode_string)
        except Exception:
            return False
    
    def get_rooch_address(self) -> RoochAddress:
        """Get the Rooch address using the correct PublicKey -> BitcoinAddress -> RoochAddress flow.
        This ensures consistency with Rust and TypeScript SDKs.
        """
        from ..address.bitcoin import BitcoinAddress
        
        # Get compressed public key for Bitcoin address generation
        public_key_uncompressed = self.get_public_key()
        
        # Convert to compressed format (33 bytes: 0x02/0x03 + X coordinate)
        # Extract X coordinate (bytes 1-32 from uncompressed format)
        x_coord = public_key_uncompressed[1:33]
        
        # Determine Y coordinate parity from full uncompressed key
        y_coord = public_key_uncompressed[33:65]
        y_int = int.from_bytes(y_coord, byteorder='big')
        
        # Compressed public key: 0x02 if Y is even, 0x03 if Y is odd
        if y_int % 2 == 0:
            compressed_public_key = b'\x02' + x_coord
        else:
            compressed_public_key = b'\x03' + x_coord
        
        # Generate Bitcoin address from compressed public key (Taproot P2TR)
        bitcoin_address = BitcoinAddress.from_taproot_public_key(compressed_public_key)
        
        # Convert Bitcoin address to Rooch address string, then to RoochAddress object
        rooch_address_str = bitcoin_address.to_rooch_address()
        return RoochAddress.from_hex(rooch_address_str)
    
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