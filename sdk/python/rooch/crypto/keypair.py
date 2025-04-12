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
        
        # Derive private key from seed using HMAC-SHA256
        key = hmac.new(b'ROOCH_KEYPAIR_SEED', seed, hashlib.sha256).digest()
        return cls(key)
    
    @classmethod
    def generate(cls) -> 'KeyPair':
        """Generate a new random key pair
        
        Returns:
            KeyPair instance
        """
        return cls()
    
    def get_public_key(self) -> bytes:
        """Get the public key
        
        Returns:
            Public key as bytes
        """
        return self._key.public_key().export_key(format='raw')
    
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
            RoochAddress instance
        """
        public_key = self.get_public_key()
        # Hash the public key to get the address
        address_bytes = hashlib.sha256(public_key).digest()[:20]  # Take first 20 bytes
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