#!/usr/bin/env python3
# Copyright (c) Rooch Network
# SPDX-License-Identifier: Apache-2.0

from abc import ABC, abstractmethod
from typing import Dict, Optional, Union

from ..address.rooch import RoochAddress
from .keypair import KeyPair


class Signer(ABC):
    """Abstract base class for transaction signers"""
    
    @abstractmethod
    def get_address(self) -> str:
        """Get the address associated with this signer
        
        Returns:
            Address as hex string
        """
        pass
    
    @abstractmethod
    def get_public_key(self) -> bytes:
        """Get the public key
        
        Returns:
            Public key as bytes
        """
        pass
    
    @abstractmethod
    def get_public_key_hex(self) -> str:
        """Get the public key as hex
        
        Returns:
            Public key as hex string
        """
        pass
    
    @abstractmethod
    def sign(self, message: Union[str, bytes]) -> bytes:
        """Sign a message
        
        Args:
            message: Message to sign
            
        Returns:
            Signature as bytes
        """
        pass
    
    @abstractmethod
    def sign_hex(self, message: Union[str, bytes]) -> str:
        """Sign a message and return the signature as hex
        
        Args:
            message: Message to sign
            
        Returns:
            Signature as hex string
        """
        pass
    
    @abstractmethod
    def sign_transaction(self, transaction) -> Dict[str, any]:
        """Sign a transaction
        
        Args:
            transaction: Transaction to sign
            
        Returns:
            Transaction authentication data
        """
        pass
    
    @abstractmethod
    def get_rooch_address(self) -> RoochAddress:
        """Get the Rooch address associated with this signer
        
        Returns:
            RoochAddress instance
        """
        pass


class RoochSigner(Signer):
    """Implementation of Signer using a KeyPair"""
    
    def __init__(self, keypair: KeyPair):
        """Initialize a signer with a key pair
        
        Args:
            keypair: KeyPair instance
        """
        self._keypair = keypair
    
    @classmethod
    def from_private_key(cls, private_key: Union[str, bytes]) -> 'RoochSigner':
        """Create a signer from a private key
        
        Args:
            private_key: Private key as hex string or bytes
            
        Returns:
            RoochSigner instance
        """
        return cls(KeyPair.from_private_key(private_key))
    
    @classmethod
    def from_seed(cls, seed: Union[str, bytes]) -> 'RoochSigner':
        """Create a signer from a seed
        
        Args:
            seed: Seed as string or bytes
            
        Returns:
            RoochSigner instance
        """
        return cls(KeyPair.from_seed(seed))
    
    @classmethod
    def generate(cls) -> 'RoochSigner':
        """Generate a new random signer
        
        Returns:
            RoochSigner instance
        """
        return cls(KeyPair.generate())
    
    def get_address(self) -> str:
        """Get the address associated with this signer
        
        Returns:
            Address as hex string
        """
        return str(self.get_rooch_address())
    
    def get_public_key(self) -> bytes:
        """Get the public key
        
        Returns:
            Public key as bytes
        """
        return self._keypair.get_public_key()
    
    def get_public_key_hex(self) -> str:
        """Get the public key as hex
        
        Returns:
            Public key as hex string
        """
        return self._keypair.get_public_key_hex()
    
    def sign(self, message: Union[str, bytes]) -> bytes:
        """Sign a message
        
        Args:
            message: Message to sign
            
        Returns:
            Signature as bytes
        """
        return self._keypair.sign(message)
    
    def sign_hex(self, message: Union[str, bytes]) -> str:
        """Sign a message and return the signature as hex
        
        Args:
            message: Message to sign
            
        Returns:
            Signature as hex string
        """
        return self._keypair.sign_hex(message)
    
    def sign_transaction(self, transaction) -> Dict[str, any]:
        """Sign a transaction
        
        Args:
            transaction: Transaction to sign
            
        Returns:
            Transaction authentication data
        """
        # Get the encoded transaction data
        tx_bytes = transaction.encode_data()
        
        # Sign the transaction data
        signature = self.sign(tx_bytes)
        
        # Create the authentication data
        auth_data = {
            "account_addr": self.get_address(),
            "public_key": self.get_public_key_hex(),
            "signature": self.sign_hex(tx_bytes),
        }
        
        return auth_data
    
    def get_rooch_address(self) -> RoochAddress:
        """Get the Rooch address associated with this signer
        
        Returns:
            RoochAddress instance
        """
        return self._keypair.get_rooch_address()
    
    def get_keypair(self) -> KeyPair:
        """Get the underlying key pair
        
        Returns:
            KeyPair instance
        """
        return self._keypair