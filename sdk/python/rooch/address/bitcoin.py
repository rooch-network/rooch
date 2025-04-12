#!/usr/bin/env python3
# Copyright (c) Rooch Network
# SPDX-License-Identifier: Apache-2.0

from enum import Enum
from typing import Optional, Union
import base58
import hashlib
import re


class BitcoinNetworkType(str, Enum):
    """Bitcoin network types"""
    
    MAINNET = "mainnet"
    TESTNET = "testnet"
    REGTEST = "regtest"


class BitcoinAddress:
    """Class for handling Bitcoin addresses"""
    
    # Address format regex patterns
    P2PKH_MAINNET_PATTERN = re.compile(r"^1[a-km-zA-HJ-NP-Z1-9]{25,34}$")
    P2PKH_TESTNET_PATTERN = re.compile(r"^[mn][a-km-zA-HJ-NP-Z1-9]{25,34}$")
    P2SH_MAINNET_PATTERN = re.compile(r"^3[a-km-zA-HJ-NP-Z1-9]{25,34}$")
    P2SH_TESTNET_PATTERN = re.compile(r"^2[a-km-zA-HJ-NP-Z1-9]{25,34}$")
    P2WPKH_MAINNET_PATTERN = re.compile(r"^bc1[ac-hj-np-z02-9]{39,59}$")
    P2WPKH_TESTNET_PATTERN = re.compile(r"^tb1[ac-hj-np-z02-9]{39,59}$")
    
    def __init__(self, address: str, network: Union[BitcoinNetworkType, str] = BitcoinNetworkType.MAINNET):
        """Initialize a Bitcoin address
        
        Args:
            address: Bitcoin address string
            network: Bitcoin network type
            
        Raises:
            ValueError: If the address is invalid
        """
        self._address = address
        
        # Convert string network type to enum if needed
        if isinstance(network, str):
            try:
                self._network = BitcoinNetworkType(network)
            except ValueError:
                raise ValueError(f"Invalid Bitcoin network type: {network}")
        else:
            self._network = network
        
        # Validate the address
        if not self.is_valid():
            raise ValueError(f"Invalid Bitcoin address: {address} for network: {self._network}")
    
    @staticmethod
    def validate_address(address: str, network: Union[BitcoinNetworkType, str] = BitcoinNetworkType.MAINNET) -> bool:
        """Check if a string is a valid Bitcoin address
        
        Args:
            address: Address string to check
            network: Bitcoin network type
            
        Returns:
            True if the address is valid
        """
        # Convert string network type to enum if needed
        if isinstance(network, str):
            try:
                network = BitcoinNetworkType(network)
            except ValueError:
                return False
        
        # Check basic format first based on patterns
        if network == BitcoinNetworkType.MAINNET:
            # Check mainnet patterns
            is_pattern_match = (
                BitcoinAddress.P2PKH_MAINNET_PATTERN.match(address) or
                BitcoinAddress.P2SH_MAINNET_PATTERN.match(address) or
                BitcoinAddress.P2WPKH_MAINNET_PATTERN.match(address)
            )
        else:  # TESTNET or REGTEST
            # Check testnet patterns
            is_pattern_match = (
                BitcoinAddress.P2PKH_TESTNET_PATTERN.match(address) or
                BitcoinAddress.P2SH_TESTNET_PATTERN.match(address) or
                BitcoinAddress.P2WPKH_TESTNET_PATTERN.match(address)
            )
        
        if not is_pattern_match:
            return False
            
        # For Base58 addresses (P2PKH, P2SH), validate checksum
        # Bech32 addresses would need a separate validation
        try:
            if (network == BitcoinNetworkType.MAINNET and 
                (BitcoinAddress.P2PKH_MAINNET_PATTERN.match(address) or 
                 BitcoinAddress.P2SH_MAINNET_PATTERN.match(address))):
                # Validate checksum for mainnet addresses
                return BitcoinAddress._validate_checksum_static(address)
            
            elif (network != BitcoinNetworkType.MAINNET and 
                  (BitcoinAddress.P2PKH_TESTNET_PATTERN.match(address) or 
                   BitcoinAddress.P2SH_TESTNET_PATTERN.match(address))):
                # Validate checksum for testnet addresses
                return BitcoinAddress._validate_checksum_static(address)
                
            # For Bech32 addresses, we've already validated the pattern
            return True
        except Exception:
            return False
            
    @staticmethod
    def _validate_checksum_static(address: str) -> bool:
        """Validate the checksum of a base58-encoded address
        
        Args:
            address: Address to validate
            
        Returns:
            True if the checksum is valid
        """
        try:
            # Decode the address
            decoded = base58.b58decode(address)
            
            # Check the length (address data + 4-byte checksum)
            if len(decoded) < 5:
                return False
            
            # Split the address data and checksum
            addr_data = decoded[:-4]
            checksum = decoded[-4:]
            
            # Compute the checksum
            h = hashlib.sha256(hashlib.sha256(addr_data).digest()).digest()
            computed_checksum = h[:4]
            
            # Compare the checksums
            return checksum == computed_checksum
        except Exception:
            return False
    
    @staticmethod
    def from_public_key(public_key: Union[str, bytes], mainnet: bool = True) -> 'BitcoinAddress':
        """Create a Bitcoin address from a public key
        
        Args:
            public_key: Public key as hex string or bytes
            mainnet: True for mainnet, False for testnet
            
        Returns:
            BitcoinAddress instance
            
        Raises:
            ValueError: If the public key is invalid
        """
        import hashlib
        import base58
        
        # Convert hex string to bytes if needed
        if isinstance(public_key, str):
            if public_key.startswith("0x"):
                public_key = public_key[2:]
            public_key = bytes.fromhex(public_key)
        
        # Hash the public key (RIPEMD160 of SHA256)
        sha256_hash = hashlib.sha256(public_key).digest()
        ripemd160_hash = hashlib.new('ripemd160')
        ripemd160_hash.update(sha256_hash)
        hash160 = ripemd160_hash.digest()
        
        # Add network version byte (0x00 for mainnet, 0x6f for testnet)
        version_byte = b'\x00' if mainnet else b'\x6f'
        payload = version_byte + hash160
        
        # Calculate checksum (first 4 bytes of double SHA256)
        checksum = hashlib.sha256(hashlib.sha256(payload).digest()).digest()[:4]
        
        # Combine payload and checksum and encode as base58
        address_bytes = payload + checksum
        address = base58.b58encode(address_bytes).decode('utf-8')
        
        network = BitcoinNetworkType.MAINNET if mainnet else BitcoinNetworkType.TESTNET
        return BitcoinAddress(address, network)
    
    def to_string(self) -> str:
        """Return the address as a string
        
        Returns:
            Address string
        """
        return self._address
    
    def is_valid(self) -> bool:
        """Check if the Bitcoin address is valid for the specified network
        
        Returns:
            True if the address is valid
        """
        # Check format based on address type and network
        if self._network == BitcoinNetworkType.MAINNET:
            if self.P2PKH_MAINNET_PATTERN.match(self._address):
                return self._validate_checksum()
            elif self.P2SH_MAINNET_PATTERN.match(self._address):
                return self._validate_checksum()
            elif self.P2WPKH_MAINNET_PATTERN.match(self._address):
                # For bech32 addresses, a more complex validation would be needed
                # For simplicity, we just check the pattern
                return True
        else:  # TESTNET or REGTEST
            if self.P2PKH_TESTNET_PATTERN.match(self._address):
                return self._validate_checksum()
            elif self.P2SH_TESTNET_PATTERN.match(self._address):
                return self._validate_checksum()
            elif self.P2WPKH_TESTNET_PATTERN.match(self._address):
                # For bech32 addresses, a more complex validation would be needed
                # For simplicity, we just check the pattern
                return True
        
        return False
    
    def _validate_checksum(self) -> bool:
        """Validate the checksum of a base58-encoded address
        
        Returns:
            True if the checksum is valid
        """
        try:
            # Decode the address
            decoded = base58.b58decode(self._address)
            
            # Check the length (address data + 4-byte checksum)
            if len(decoded) < 5:
                return False
            
            # Split the address data and checksum
            addr_data = decoded[:-4]
            checksum = decoded[-4:]
            
            # Compute the checksum
            h = hashlib.sha256(hashlib.sha256(addr_data).digest()).digest()
            computed_checksum = h[:4]
            
            # Compare the checksums
            return checksum == computed_checksum
        except Exception:
            return False
    
    def __str__(self) -> str:
        """Return the address as a string"""
        return self._address
    
    def __repr__(self) -> str:
        """Return the string representation of the address"""
        return f"BitcoinAddress('{self._address}', '{self._network}')"
    
    def __eq__(self, other: object) -> bool:
        """Check if two addresses are equal"""
        if not isinstance(other, BitcoinAddress):
            return False
        return self._address == other._address and self._network == other._network
    
    def __hash__(self) -> int:
        """Return hash value for the address"""
        return hash((self._address, self._network))
    
    @property
    def address(self) -> str:
        """Get the address string
        
        Returns:
            Address string
        """
        return self._address
    
    @property
    def network(self) -> BitcoinNetworkType:
        """Get the network type
        
        Returns:
            Network type
        """
        return self._network
    
    def is_p2pkh(self) -> bool:
        """Check if the address is a P2PKH address
        
        Returns:
            True if the address is P2PKH
        """
        if self._network == BitcoinNetworkType.MAINNET:
            return bool(self.P2PKH_MAINNET_PATTERN.match(self._address))
        else:  # TESTNET or REGTEST
            return bool(self.P2PKH_TESTNET_PATTERN.match(self._address))
    
    def is_p2sh(self) -> bool:
        """Check if the address is a P2SH address
        
        Returns:
            True if the address is P2SH
        """
        if self._network == BitcoinNetworkType.MAINNET:
            return bool(self.P2SH_MAINNET_PATTERN.match(self._address))
        else:  # TESTNET or REGTEST
            return bool(self.P2SH_TESTNET_PATTERN.match(self._address))
    
    def is_bech32(self) -> bool:
        """Check if the address is a Bech32 address
        
        Returns:
            True if the address is Bech32
        """
        if self._network == BitcoinNetworkType.MAINNET:
            return bool(self.P2WPKH_MAINNET_PATTERN.match(self._address))
        else:  # TESTNET or REGTEST
            return bool(self.P2WPKH_TESTNET_PATTERN.match(self._address))


def is_bitcoin_address(address: str, network: Union[BitcoinNetworkType, str] = BitcoinNetworkType.MAINNET) -> bool:
    """Check if a string is a valid Bitcoin address for the specified network
    
    Args:
        address: Address string to check
        network: Bitcoin network type
        
    Returns:
        True if the address is valid
    """
    try:
        BitcoinAddress(address, network)
        return True
    except ValueError:
        return False