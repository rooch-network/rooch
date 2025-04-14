#!/usr/bin/env python3
# Copyright (c) Rooch Network
# SPDX-License-Identifier: Apache-2.0

from enum import IntEnum
from typing import Any, Dict, Union

from ..utils.hex import from_hex, to_hex
from .move.move_types import MoveActionArgument, MoveAction
from .auth.auth_types import TransactionAuthenticator
from ..bcs.serializer import BcsSerializer, Serializable, BcsDeserializer, Deserializable


class TransactionType(IntEnum):
    """Types of Rooch transactions"""
    
    BITCOIN_MOVE_ACTION = 0
    ETHEREUM_MOVE_ACTION = 1
    MOVE_ACTION = 2
    MOVE_MODULE_TRANSACTION = 3
    BITCOIN_BINDING = 4


class TransactionData(Serializable, Deserializable):
    """Transaction data for Rooch transactions"""
    
    def __init__(
        self,
        tx_type: TransactionType,
        tx_arg: Union[MoveActionArgument, bytes],
        sequence_number: Union[int, str],
        max_gas_amount: Union[int, str] = 1000000,
        gas_unit_price: Union[int, str] = 1,
        expiration_timestamp_secs: Union[int, str] = 0,
        chain_id: int = 42
    ):
        """
        Args:
            tx_type: Transaction type
            tx_arg: Move action argument or module bytes
            sequence_number: Transaction sequence number
            max_gas_amount: Maximum gas amount
            gas_unit_price: Gas unit price
            expiration_timestamp_secs: Expiration timestamp in seconds
            chain_id: Chain ID
        """
        self.tx_type = tx_type
        self.tx_arg = tx_arg
        self.sequence_number = int(sequence_number)
        self.max_gas_amount = int(max_gas_amount)
        self.gas_unit_price = int(gas_unit_price)
        self.expiration_timestamp_secs = int(expiration_timestamp_secs)
        self.chain_id = chain_id

    def serialize(self, serializer: BcsSerializer):
        """Serialize the transaction data."""
        serializer.u8(self.tx_type.value)
        if isinstance(self.tx_arg, MoveActionArgument):
            serializer.struct(self.tx_arg)
        else:
            serializer.bytes(self.tx_arg)
        serializer.u64(self.sequence_number)
        serializer.u64(self.max_gas_amount)
        serializer.u64(self.gas_unit_price)
        serializer.u64(self.expiration_timestamp_secs)
        serializer.u8(self.chain_id)

    @staticmethod
    def deserialize(deserializer: BcsDeserializer) -> 'TransactionData':
        """Deserialize a transaction data."""
        tx_type = TransactionType(deserializer.u8())
        if tx_type == TransactionType.MOVE_ACTION:
            tx_arg = MoveActionArgument.deserialize(deserializer)
        else:
            tx_arg = deserializer.bytes()
        sequence_number = deserializer.u64()
        max_gas_amount = deserializer.u64()
        gas_unit_price = deserializer.u64()
        expiration_timestamp_secs = deserializer.u64()
        chain_id = deserializer.u8()
        return TransactionData(
            tx_type=tx_type,
            tx_arg=tx_arg,
            sequence_number=sequence_number,
            max_gas_amount=max_gas_amount,
            gas_unit_price=gas_unit_price,
            expiration_timestamp_secs=expiration_timestamp_secs,
            chain_id=chain_id
        )
    
    def to_dict(self) -> Dict[str, Any]:
        """Convert to dictionary
        
        Returns:
            Dictionary representation
        """
        result = {
            "tx_type": self.tx_type,
            "sequence_number": str(self.sequence_number),
            "max_gas_amount": str(self.max_gas_amount),
            "gas_unit_price": str(self.gas_unit_price),
            "expiration_timestamp_secs": str(self.expiration_timestamp_secs),
            "chain_id": self.chain_id
        }
        
        if isinstance(self.tx_arg, MoveActionArgument):
            result["tx_arg"] = self.tx_arg.to_dict()
        else:
            result["tx_arg"] = to_hex(self.tx_arg)
            
        return result
    
    @classmethod
    def from_dict(cls, data: Dict[str, Any]) -> 'TransactionData':
        """Create from dictionary
        
        Args:
            data: Dictionary representation
            
        Returns:
            TransactionData instance
        """
        tx_type = data.get("tx_type", TransactionType.MOVE_ACTION)
        tx_arg_data = data.get("tx_arg", {})
        
        if tx_type == TransactionType.MOVE_ACTION:
            tx_arg = MoveActionArgument.from_dict(tx_arg_data)
        else:
            tx_arg = from_hex(tx_arg_data)
        
        return cls(
            tx_type=tx_type,
            tx_arg=tx_arg,
            sequence_number=data.get("sequence_number", "0"),
            max_gas_amount=data.get("max_gas_amount", "1000000"),
            gas_unit_price=data.get("gas_unit_price", "1"),
            expiration_timestamp_secs=data.get("expiration_timestamp_secs", "0"),
            chain_id=data.get("chain_id", 42)
        )


class SignedTransaction(Serializable, Deserializable):
    """Signed transaction ready for submission"""
    
    def __init__(self, tx_data: TransactionData, authenticator: TransactionAuthenticator):
        """
        Args:
            tx_data: Transaction data
            authenticator: Transaction authenticator
        """
        self.tx_data = tx_data
        self.authenticator = authenticator

    def serialize(self, serializer: BcsSerializer):
        """Serialize the signed transaction."""
        serializer.struct(self.tx_data)
        serializer.struct(self.authenticator)

    @staticmethod
    def deserialize(deserializer: BcsDeserializer) -> 'SignedTransaction':
        """Deserialize a signed transaction."""
        tx_data = TransactionData.deserialize(deserializer)
        authenticator = TransactionAuthenticator.deserialize(deserializer)
        return SignedTransaction(tx_data=tx_data, authenticator=authenticator)
    
    def to_dict(self) -> Dict[str, Any]:
        """Convert to dictionary
        
        Returns:
            Dictionary representation
        """
        return {
            "tx_data": self.tx_data.to_dict(),
            "authenticator": self.authenticator.to_dict()
        }
    
    @classmethod
    def from_dict(cls, data: Dict[str, Any]) -> 'SignedTransaction':
        """Create from dictionary
        
        Args:
            data: Dictionary representation
            
        Returns:
            SignedTransaction instance
        """
        return cls(
            tx_data=TransactionData.from_dict(data.get("tx_data", {})),
            authenticator=TransactionAuthenticator.from_dict(data.get("authenticator", {}))
        ) 