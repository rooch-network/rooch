#!/usr/bin/env python3
# Copyright (c) Rooch Network
# SPDX-License-Identifier: Apache-2.0

from enum import IntEnum
from typing import Any, Dict, Optional, Union

from ...utils.hex import ensure_hex_prefix, to_hex, from_hex
from ...bcs.serializer import BcsSerializer, Serializable, BcsDeserializer, Deserializable
from ...address.rooch import RoochAddress


# AuthValidator IDs (should match Rust BuiltinAuthValidator flags)
SESSION_AUTH_VALIDATOR_ID = 0  # Example, update as per Rust
BITCOIN_AUTH_VALIDATOR_ID = 1  # Example, update as per Rust
# Add more as needed


class TransactionAuthenticator(Serializable, Deserializable):
    """Authentication data for transactions, compatible with Rust Authenticator struct"""
    def __init__(self, auth_validator_id: int, payload: bytes):
        self.auth_validator_id = auth_validator_id
        self.payload = payload

    @classmethod
    def session(cls, signature: Union[str, bytes]):
        if isinstance(signature, str):
            sig_bytes = from_hex(ensure_hex_prefix(signature))
        else:
            sig_bytes = signature
        return cls(auth_validator_id=SESSION_AUTH_VALIDATOR_ID, payload=sig_bytes)

    @classmethod
    def bitcoin(cls, payload: bytes):
        return cls(auth_validator_id=BITCOIN_AUTH_VALIDATOR_ID, payload=payload)

    def serialize(self, serializer: BcsSerializer):
        serializer.u64(self.auth_validator_id)
        serializer.bytes(self.payload)

    @staticmethod
    def deserialize(deserializer: BcsDeserializer) -> 'TransactionAuthenticator':
        auth_validator_id = deserializer.u64()
        payload = deserializer.bytes()
        return TransactionAuthenticator(auth_validator_id, payload)

    def to_dict(self) -> Dict[str, Any]:
        return {
            "auth_validator_id": self.auth_validator_id,
            "payload": to_hex(self.payload)
        }


class AuthPayload(Serializable, Deserializable):
    """Authentication payload for transaction signatures"""
    
    def __init__(self, public_key: Union[str, bytes], message: Union[str, bytes], signature: Union[str, bytes], address: Optional[Union[str, RoochAddress]] = None):
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
            
        # Normalize address
        if isinstance(address, RoochAddress):
            self.address = str(address)
        else:
            self.address = address
    
    def serialize(self, serializer: BcsSerializer):
        """Serialize the authentication payload."""
        serializer.bytes(self.public_key)
        serializer.bytes(self.message)
        serializer.bytes(self.signature)
        if self.address:
            serializer.bool(True)
            serializer.str(self.address)
        else:
            serializer.bool(False)

    @staticmethod
    def deserialize(deserializer: BcsDeserializer) -> 'AuthPayload':
        """Deserialize an authentication payload."""
        public_key = deserializer.bytes()
        message = deserializer.bytes()
        signature = deserializer.bytes()
        has_address = deserializer.bool()
        address = deserializer.str() if has_address else None
        return AuthPayload(
            public_key=public_key,
            message=message,
            signature=signature,
            address=address
        )
    
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