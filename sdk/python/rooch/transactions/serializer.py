#!/usr/bin/env python3
# Copyright (c) Rooch Network
# SPDX-License-Identifier: Apache-2.0

from typing import Any, Dict, List, Optional, Union

from ..bcs.serializer import BcsSerializer, BcsSerializationError
from .types import (
    AuthenticatorType,
    FunctionArgument,
    MoveAction,
    MoveActionArgument,
    SignedTransaction,
    TransactionArgument,
    TransactionAuthenticator,
    TransactionData,
    TransactionType
)
from ..utils.hex import from_hex, to_hex


class TxSerializer:
    """Serializer for Rooch transactions"""
    
    @staticmethod
    def encode_transaction_data(tx_data: TransactionData) -> bytes:
        """Encode transaction data in BCS format
        
        Args:
            tx_data: Transaction data
            
        Returns:
            Encoded transaction bytes
            
        Raises:
            BcsSerializationError: If serialization fails
        """
        try:
            # Serialize tx_type
            result = BcsSerializer.serialize_u8(tx_data.tx_type)
            
            # Serialize tx_arg based on type
            if tx_data.tx_type == TransactionType.MOVE_ACTION:
                result += TxSerializer._encode_move_action(tx_data.tx_arg)
            elif tx_data.tx_type == TransactionType.MOVE_MODULE_TRANSACTION:
                # For module bytecode, encode as bytes
                if isinstance(tx_data.tx_arg, str):
                    module_bytes = from_hex(tx_data.tx_arg)
                else:
                    module_bytes = tx_data.tx_arg
                result += BcsSerializer.serialize_bytes(module_bytes)
            elif tx_data.tx_type == TransactionType.BITCOIN_BINDING:
                # For Bitcoin binding, encode as string
                result += BcsSerializer.serialize_string(tx_data.tx_arg)
            else:
                # Default fallback - try to encode as string
                result += BcsSerializer.serialize_string(str(tx_data.tx_arg))
            
            # Serialize sequence number
            result += BcsSerializer.serialize_u64(tx_data.sequence_number)
            
            # Serialize gas parameters
            result += BcsSerializer.serialize_u64(tx_data.max_gas_amount)
            result += BcsSerializer.serialize_u64(tx_data.gas_unit_price)
            
            # Serialize expiration
            result += BcsSerializer.serialize_u64(tx_data.expiration_timestamp_secs)
            
            # Serialize chain ID
            result += BcsSerializer.serialize_u8(tx_data.chain_id)
            
            return result
        except Exception as e:
            raise BcsSerializationError(f"Failed to serialize transaction data: {str(e)}")
    
    @staticmethod
    def _encode_move_action(action_arg: MoveActionArgument) -> bytes:
        """Encode a Move action argument
        
        Args:
            action_arg: Move action argument
            
        Returns:
            Encoded bytes
            
        Raises:
            BcsSerializationError: If serialization fails
        """
        # Serialize action type
        result = BcsSerializer.serialize_u8(action_arg.action)
        
        if action_arg.action == MoveAction.FUNCTION:
            # Encode function call
            func_arg = action_arg.args
            
            # Serialize function_id
            result += BcsSerializer.serialize_string(func_arg.function_id)
            
            # Serialize type arguments
            result += BcsSerializer.serialize_vector(
                func_arg.ty_args,
                BcsSerializer.serialize_string
            )
            
            # Serialize arguments
            result += BcsSerializer.serialize_vector(
                func_arg.args,
                TxSerializer._encode_transaction_argument
            )
        else:  # SCRIPT
            # Encode script
            result += BcsSerializer.serialize_string(action_arg.args)
        
        return result
    
    @staticmethod
    def _encode_transaction_argument(arg: TransactionArgument) -> bytes:
        """Encode a transaction argument
        
        Args:
            arg: Transaction argument
            
        Returns:
            Encoded bytes
            
        Raises:
            BcsSerializationError: If serialization fails
        """
        # Serialize type tag
        result = BcsSerializer.serialize_u8(arg.type_tag)
        
        # Serialize value based on type tag
        if arg.type_tag == 0:  # u8
            result += BcsSerializer.serialize_u8(arg.value)
        elif arg.type_tag == 1:  # u16
            result += BcsSerializer.serialize_u16(arg.value)
        elif arg.type_tag == 2:  # u32
            result += BcsSerializer.serialize_u32(arg.value)
        elif arg.type_tag == 3:  # u64
            # Handle string representation for large numbers
            if isinstance(arg.value, str):
                result += BcsSerializer.serialize_u64(int(arg.value))
            else:
                result += BcsSerializer.serialize_u64(arg.value)
        elif arg.type_tag == 4:  # u128
            # Handle string representation for large numbers
            if isinstance(arg.value, str):
                result += BcsSerializer.serialize_u128(int(arg.value))
            else:
                result += BcsSerializer.serialize_u128(arg.value)
        elif arg.type_tag == 5:  # u256
            # Handle string representation for large numbers
            if isinstance(arg.value, str):
                result += BcsSerializer.serialize_u256(int(arg.value))
            else:
                result += BcsSerializer.serialize_u256(arg.value)
        elif arg.type_tag == 6:  # bool
            result += BcsSerializer.serialize_bool(arg.value)
        elif arg.type_tag == 7:  # address
            # Handle address as hex string
            from ..utils.hex import from_hex, ensure_hex_prefix
            addr_bytes = from_hex(ensure_hex_prefix(arg.value))
            result += BcsSerializer.serialize_fixed_bytes(addr_bytes)
        elif arg.type_tag == 8:  # string
            result += BcsSerializer.serialize_string(arg.value)
        elif arg.type_tag == 9:  # vector
            # Handle vector with type information
            vector_info = arg.value
            element_type = vector_info.get("type")
            values = vector_info.get("value", [])
            
            # Create a vector of transaction arguments
            vector_args = []
            for val in values:
                # Determine element type tag
                if element_type == "u8":
                    vector_args.append(TransactionArgument(0, val))
                elif element_type == "u16":
                    vector_args.append(TransactionArgument(1, val))
                elif element_type == "u32":
                    vector_args.append(TransactionArgument(2, val))
                elif element_type == "u64":
                    vector_args.append(TransactionArgument(3, val))
                elif element_type == "u128":
                    vector_args.append(TransactionArgument(4, val))
                elif element_type == "u256":
                    vector_args.append(TransactionArgument(5, val))
                elif element_type == "bool":
                    vector_args.append(TransactionArgument(6, val))
                elif element_type == "address":
                    vector_args.append(TransactionArgument(7, val))
                elif element_type == "string":
                    vector_args.append(TransactionArgument(8, val))
                elif element_type == "objectId":
                    vector_args.append(TransactionArgument(10, val))
                else:
                    raise BcsSerializationError(f"Unsupported vector element type: {element_type}")
            
            # Serialize vector of arguments
            result += BcsSerializer.serialize_vector(
                vector_args,
                TxSerializer._encode_transaction_argument
            )
        elif arg.type_tag == 10:  # objectId
            # Handle object ID as string
            result += BcsSerializer.serialize_string(arg.value)
        else:
            raise BcsSerializationError(f"Unsupported argument type tag: {arg.type_tag}")
        
        return result
    
    @staticmethod
    def encode_signed_transaction(signed_tx: SignedTransaction) -> bytes:
        """Encode a signed transaction for submission
        
        Args:
            signed_tx: Signed transaction
            
        Returns:
            Encoded transaction bytes
            
        Raises:
            BcsSerializationError: If serialization fails
        """
        try:
            tx_data_bytes = TxSerializer.encode_transaction_data(signed_tx.tx_data)
            auth = signed_tx.authenticator
            
            # Encode authenticator
            result = BcsSerializer.serialize_string(auth.account_addr)
            result += BcsSerializer.serialize_u8(auth.auth_key.auth_type)
            result += BcsSerializer.serialize_bytes(auth.auth_key.public_key)
            result += BcsSerializer.serialize_bytes(auth.signature)
            
            # Combine tx_data and authenticator
            return tx_data_bytes + result
        except Exception as e:
            raise BcsSerializationError(f"Failed to serialize signed transaction: {str(e)}")
    
    @staticmethod
    def encode_transaction_for_submission(signed_tx: SignedTransaction) -> str:
        """Encode a signed transaction as hex for submission
        
        Args:
            signed_tx: Signed transaction
            
        Returns:
            Hex-encoded transaction data
            
        Raises:
            BcsSerializationError: If serialization fails
        """
        tx_bytes = TxSerializer.encode_signed_transaction(signed_tx)
        return to_hex(tx_bytes)