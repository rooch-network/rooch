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
    TransactionAuthenticator,
    TransactionData,
    TransactionType,
    ModuleId,
    FunctionId,
    TypeTag,
    StructTag
)
from ..utils.hex import from_hex, to_hex
from ..address.rooch import RoochAddress


class TxSerializer:
    """Serializer for Rooch transactions"""
    
    @staticmethod
    def encode_transaction_data(tx_data: TransactionData) -> bytes:
        """Encode transaction data in BCS format matching Rust RoochTransactionData
        
        Args:
            tx_data: Transaction data object
            
        Returns:
            Encoded transaction bytes
            
        Raises:
            BcsSerializationError: If serialization fails
        """
        try:
            # Sequence: sender, sequence_number, chain_id, max_gas_amount, action
            
            # 1. sender (RoochAddress)
            # Assuming RoochAddress needs conversion to bytes first
            sender_addr = RoochAddress.from_hex(tx_data.sender)
            sender_bytes = sender_addr.to_bytes() # Assuming this method exists
            # Assuming address is BCS serialized as fixed bytes (like AccountAddress)
            # Need to confirm the exact BCS format for RoochAddress
            result = BcsSerializer.serialize_fixed_bytes(sender_bytes) 
            
            # 2. sequence_number (u64)
            result += BcsSerializer.serialize_u64(tx_data.sequence_number)
            
            # 3. chain_id (u64)
            result += BcsSerializer.serialize_u64(tx_data.chain_id)
            
            # 4. max_gas_amount (u64)
            result += BcsSerializer.serialize_u64(tx_data.max_gas_amount)
            
            # 5. action (MoveAction)
            result += TxSerializer._encode_move_action(tx_data.action)
            
            return result
        except Exception as e:
            # Add more context to the error
            raise BcsSerializationError(f"Failed to serialize transaction data: {e}. Data: {tx_data}") from e
    
    @staticmethod
    def _encode_move_action(action_arg: MoveActionArgument) -> bytes:
        """Encode a Move action argument matching Rust MoveAction enum
        
        Args:
            action_arg: MoveActionArgument object
            
        Returns:
            Encoded bytes
            
        Raises:
            BcsSerializationError: If serialization fails
        """
        # Serialize variant index (Script=0, Function=1, ModuleBundle=2)
        # Assuming the Rust MoveAction enum variant index uses u8 (common for simple enums)
        result = BcsSerializer.serialize_u8(action_arg.action.value) # Use u8 instead of u32
        
        if action_arg.action == MoveAction.SCRIPT:
            # TODO: Implement _encode_script_call if needed
            # script_call = action_arg.args
            # result += TxSerializer._encode_script_call(script_call)
            raise NotImplementedError("Script action serialization not implemented")
        elif action_arg.action == MoveAction.FUNCTION:
            function_arg = action_arg.args # This is FunctionArgument type
            result += TxSerializer._encode_function_call(function_arg)
        elif action_arg.action == MoveAction.MODULE_BUNDLE:
            module_bundle = action_arg.args # This is List[bytes]
            result += BcsSerializer.serialize_vector(
                module_bundle,
                BcsSerializer.serialize_bytes
            )
        else:
            raise BcsSerializationError(f"Unsupported MoveAction type: {action_arg.action}")
        
        return result
    
    @staticmethod
    def _encode_function_call(func_arg: FunctionArgument) -> bytes:
        """Encode a FunctionCall matching Rust FunctionCall struct
        
        Args:
            func_arg: FunctionArgument object containing FunctionId and TypeTags
        
        Returns:
            Encoded bytes
        
        Raises:
            BcsSerializationError: If serialization fails
        """
        # Sequence: function_id, ty_args, args
        
        # 1. function_id (FunctionId)
        # func_arg.function_id is now a FunctionId object
        try:
            result = BcsSerializer.serialize_function_id(func_arg.function_id)
        except Exception as e:
             raise BcsSerializationError(f"Failed to serialize function_id object: {e}") from e
        
        # 2. ty_args (Vec<TypeTag>)
        # func_arg.ty_args is now a list of TypeTag objects
        try:
            result += BcsSerializer.serialize_vector(
                func_arg.ty_args,
                BcsSerializer.serialize_type_tag
            )
        except Exception as e:
             raise BcsSerializationError(f"Failed to serialize ty_args: {e}") from e
        
        # 3. args (Vec<Vec<u8>>) - BCS encode each raw value
        encoded_args_list = []
        try:
            for raw_arg_value in func_arg.args:
                # ... (argument encoding logic remains the same) ...
                if isinstance(raw_arg_value, bool):
                    encoded_args_list.append(BcsSerializer.serialize_bool(raw_arg_value))
                elif isinstance(raw_arg_value, int):
                    encoded_args_list.append(BcsSerializer.serialize_u256(raw_arg_value))
                elif isinstance(raw_arg_value, str):
                    if raw_arg_value.startswith("0x"):
                        try:
                            addr = RoochAddress.from_hex(raw_arg_value) # Use from_hex for full addresses here
                            encoded_args_list.append(BcsSerializer.serialize_fixed_bytes(addr.to_bytes()))
                        except ValueError:
                            # Could be Object ID or other hex, treat as string for now
                            encoded_args_list.append(BcsSerializer.serialize_string(raw_arg_value))
                    else:
                        encoded_args_list.append(BcsSerializer.serialize_string(raw_arg_value))
                elif isinstance(raw_arg_value, bytes):
                    encoded_args_list.append(BcsSerializer.serialize_bytes(raw_arg_value))
                else:
                    raise BcsSerializationError(f"Unsupported argument type for BCS encoding: {type(raw_arg_value)}")
        
            result += BcsSerializer.serialize_vector(
                encoded_args_list,
                lambda x: x # Items are already bytes
            )
        except Exception as e:
             raise BcsSerializationError(f"Failed to serialize args: {e}") from e
        
        return result
    
    @staticmethod
    def _encode_authenticator(auth: TransactionAuthenticator) -> bytes:
        """Encode a TransactionAuthenticator matching Rust Authenticator struct.
        
        Args:
            auth: TransactionAuthenticator object
        
        Returns:
            Encoded bytes
        
        Raises:
            BcsSerializationError: If serialization fails
        """
        # Sequence: auth_validator_id, payload
        
        # 1. auth_validator_id (u64) - Map Python AuthenticatorType to ID
        # Assuming standard mapping: Ed25519: 0, Secp256k1: 1, Secp256r1: 2
        auth_id_map = {
            AuthenticatorType.ED25519: 0,
            AuthenticatorType.SECP256K1: 1,
            AuthenticatorType.SECP256R1: 2, # Corrected mapping
            # TODO: Add MULTISIG mappings if needed
        }
        auth_validator_id = auth_id_map.get(auth.auth_type)
        if auth_validator_id is None:
            raise BcsSerializationError(f"Unsupported authenticator type for ID mapping: {auth.auth_type}")
        
        # Revert to u64 serialization based on Rust struct definition
        result = BcsSerializer.serialize_u64(auth_validator_id)
        
        # 2. payload (Vec<u8>)
        # Payload for single-signature should be: public_key_bytes + signature_bytes
        
        # Ensure public_key is bytes
        if isinstance(auth.public_key, str):
             public_key_bytes = from_hex(auth.public_key) # Assuming hex string
        else:
             public_key_bytes = auth.public_key
             
        # Ensure signature is bytes
        if isinstance(auth.signature, str):
            signature_bytes = from_hex(auth.signature)
        else:
            signature_bytes = auth.signature
            
        # Construct payload
        payload_bytes = public_key_bytes + signature_bytes
            
        # Serialize payload as BCS bytes (Vec<u8>)
        result += BcsSerializer.serialize_bytes(payload_bytes)
        
        # Remove the old incorrect payload serialization
        # result += BcsSerializer.serialize_bytes(signature_bytes)
        
        # TODO: Verify payload for MULTISIG and potentially other auth types
        
        return result

    @staticmethod
    def encode_signed_transaction(signed_tx: SignedTransaction) -> bytes:
        """Encode a signed transaction matching Rust RoochTransaction.
        
        Args:
            signed_tx: Signed transaction object
            
        Returns:
            Encoded transaction bytes
            
        Raises:
            BcsSerializationError: If serialization fails
        """
        try:
            # Sequence: data, authenticator
            tx_data_bytes = TxSerializer.encode_transaction_data(signed_tx.tx_data)
            auth_bytes = TxSerializer._encode_authenticator(signed_tx.authenticator)
            return tx_data_bytes + auth_bytes
        except Exception as e:
            raise BcsSerializationError(f"Failed to serialize signed transaction: {e}") from e
    
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
        encoded_bytes = TxSerializer.encode_signed_transaction(signed_tx)
        return to_hex(encoded_bytes)