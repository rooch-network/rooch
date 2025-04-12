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