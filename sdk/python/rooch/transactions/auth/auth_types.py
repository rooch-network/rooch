#!/usr/bin/env python3
# Copyright (c) Rooch Network
# SPDX-License-Identifier: Apache-2.0

from enum import IntEnum
from typing import Any, Dict, Optional, Union

from ...utils.hex import ensure_hex_prefix, to_hex, from_hex
from ...bcs.serializer import BcsSerializer, Serializable, BcsDeserializer, Deserializable


class AuthenticatorType(IntEnum):
    """Types of transaction authenticators"""
    
    ED25519 = 0
    SECP256K1 = 1
    SECP256R1 = 2
    MULTI_ED25519 = 3
    MULTI_SECP256K1 = 4
    MULTI_SECP256R1 = 5


class AuthenticationKey(Serializable, Deserializable):
    """Authentication key for transactions"""
    
    def __init__(self, auth_type: AuthenticatorType, public_key: Union[str, bytes]):
        """
        Args:
            auth_type: Authentication type
            public_key: Public key (hex string or bytes)
        """
        self.auth_type = auth_type
        
        # Normalize public key
        if isinstance(public_key, str):
            self.public_key = from_hex(ensure_hex_prefix(public_key))
        else:
            self.public_key = public_key

    def serialize(self, serializer: BcsSerializer):
        """Serialize the authentication key."""
        serializer.u8(self.auth_type.value)
        serializer.bytes(self.public_key)

    @staticmethod
    def deserialize(deserializer: BcsDeserializer) -> 'AuthenticationKey':
        """Deserialize an authentication key."""
        auth_type = AuthenticatorType(deserializer.u8())
        public_key = deserializer.bytes()
        return AuthenticationKey(auth_type=auth_type, public_key=public_key)
    
    def to_dict(self) -> Dict[str, Any]:
        """Convert to dictionary
        
        Returns:
            Dictionary representation
        """
        return {
            "auth_type": self.auth_type,
            "public_key": to_hex(self.public_key)
        }


class TransactionAuthenticator(Serializable, Deserializable):
    """Authentication data for transactions"""
    
    def __init__(
        self,
        account_addr: str,
        public_key: Union[str, bytes],
        signature: Union[str, bytes],
        auth_type: AuthenticatorType = AuthenticatorType.ED25519
    ):
        """
        Args:
            account_addr: Account address
            public_key: Public key (hex string or bytes)
            signature: Signature (hex string or bytes)
            auth_type: Authentication type
        """
        self.account_addr = account_addr
        self.auth_key = AuthenticationKey(auth_type=auth_type, public_key=public_key)
        
        # Normalize signature
        if isinstance(signature, str):
            self.signature = from_hex(ensure_hex_prefix(signature))
        else:
            self.signature = signature

    def serialize(self, serializer: BcsSerializer):
        """Serialize the transaction authenticator."""
        serializer.struct(self.auth_key)
        serializer.bytes(self.signature)

    @staticmethod
    def deserialize(deserializer: BcsDeserializer) -> 'TransactionAuthenticator':
        """Deserialize a transaction authenticator."""
        auth_key = AuthenticationKey.deserialize(deserializer)
        signature = deserializer.bytes()
        return TransactionAuthenticator(
            account_addr="",  # We don't serialize/deserialize account_addr
            public_key=auth_key.public_key,
            signature=signature,
            auth_type=auth_key.auth_type
        )
    
    def to_dict(self) -> Dict[str, Any]:
        """Convert to dictionary
        
        Returns:
            Dictionary representation
        """
        return {
            "account_addr": self.account_addr,
            "auth_key": self.auth_key.to_dict(),
            "signature": to_hex(self.signature)
        }
    
    @classmethod
    def from_dict(cls, data: Dict[str, Any]) -> 'TransactionAuthenticator':
        """Create from dictionary
        
        Args:
            data: Dictionary representation
            
        Returns:
            TransactionAuthenticator instance
        """
        auth_key_data = data.get("auth_key", {})
        return cls(
            account_addr=data.get("account_addr", ""),
            public_key=auth_key_data.get("public_key", ""),
            signature=data.get("signature", ""),
            auth_type=auth_key_data.get("auth_type", AuthenticatorType.ED25519)
        )


class AuthPayload:
    """Authentication payload for transaction signatures"""
    
    def __init__(self, public_key: Union[str, bytes], message: Union[str, bytes], signature: Union[str, bytes], address: Optional[str] = None):
        """
        Args:
            public_key: Public key (hex string or bytes)
            message: Message that was signed (string or bytes)
            signature: Signature (hex string or bytes)
            address: Optional address derived from public key
        """
        # Normalize public key
        if isinstance(public_key, str):
            self.public_key = from_hex(ensure_hex_prefix(public_key))
        else:
            self.public_key = public_key
            
        # Normalize message
        if isinstance(message, str):
            self.message = message.encode('utf-8')
        else:
            self.message = message
            
        # Normalize signature
        if isinstance(signature, str):
            self.signature = from_hex(ensure_hex_prefix(signature))
        else:
            self.signature = signature
            
        self.address = address
    
    def to_dict(self) -> Dict[str, Any]:
        """Convert to dictionary
        
        Returns:
            Dictionary representation
        """
        return {
            "public_key": to_hex(self.public_key),
            "message": self.message.decode('utf-8') if isinstance(self.message, bytes) else self.message,
            "signature": to_hex(self.signature),
            "address": self.address
        }
    
    @classmethod
    def from_dict(cls, data: Dict[str, Any]) -> 'AuthPayload':
        """Create from dictionary
        
        Args:
            data: Dictionary representation
            
        Returns:
            AuthPayload instance
        """
        return cls(
            public_key=data.get("public_key", ""),
            message=data.get("message", ""),
            signature=data.get("signature", ""),
            address=data.get("address")
        ) 