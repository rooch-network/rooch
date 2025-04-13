#!/usr/bin/env python3
# Copyright (c) Rooch Network
# SPDX-License-Identifier: Apache-2.0

from typing import Any, Dict, List, Optional, Union

from ..bcs.serializer import BcsSerializer, BcsSerializationError, Serializable, Deserializable
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
    StructTag,
    AuthPayload
)
from ..utils.hex import from_hex, to_hex
from ..address.rooch import RoochAddress
from ..utils.bytes import varint_byte_num


class TxSerializer:
    """Serializer for Rooch transactions using the new BcsSerializer"""
    
    @staticmethod
    def encode_transaction_data(tx_data: TransactionData) -> bytes:
        """Encode transaction data using the new BcsSerializer instance methods.
        
        Args:
            tx_data: Transaction data object
            
        Returns:
            Encoded transaction bytes
            
        Raises:
            BcsSerializationError: If serialization fails
        """
        if not isinstance(tx_data, TransactionData):
            raise TypeError("Expected TransactionData object")

        ser = BcsSerializer()
        try:
            # Sequence: sender, sequence_number, chain_id, max_gas_amount, action
            
            # 1. sender (RoochAddress - assuming it becomes Serializable)
            sender_addr = RoochAddress.from_hex(tx_data.sender)
            ser.struct(sender_addr) # Use the generic struct serializer
            
            # 2. sequence_number (u64)
            ser.u64(tx_data.sequence_number)
            
            # 3. chain_id (u64)
            ser.u64(tx_data.chain_id)
            
            # 4. max_gas_amount (u64)
            ser.u64(tx_data.max_gas_amount)
            
            # 5. action (MoveActionArgument)
            TxSerializer._encode_move_action(ser, tx_data.action)
            
            return ser.output()
        except Exception as e:
            # Add more context to the error
            raise BcsSerializationError(f"Failed to serialize transaction data: {e}. Data: {tx_data}") from e
    
    @staticmethod
    def _encode_move_action(ser: BcsSerializer, action_arg: MoveActionArgument):
        """Encode a Move action argument using the provided serializer.
        
        Args:
            ser: BcsSerializer instance
            action_arg: MoveActionArgument object
            
        Raises:
            BcsSerializationError: If serialization fails
        """
        if not isinstance(action_arg, MoveActionArgument):
            raise TypeError("Expected MoveActionArgument object")

        # Serialize variant index (u8 for MoveAction enum)
        ser.u8(action_arg.action.value)

        action_payload = action_arg.args
        if action_arg.action == MoveAction.SCRIPT:
            raise NotImplementedError("Script action serialization not implemented")
        elif action_arg.action == MoveAction.FUNCTION:
            if not isinstance(action_payload, FunctionArgument):
                raise TypeError("Expected FunctionArgument for FUNCTION action")
            TxSerializer._encode_function_call(ser, action_payload)
        elif action_arg.action == MoveAction.MODULE_BUNDLE:
            if not isinstance(action_payload, list) or not all(isinstance(b, bytes) for b in action_payload):
                raise TypeError("Expected List[bytes] for MODULE_BUNDLE action")
            ser.sequence(action_payload, BcsSerializer.bytes)
        else:
            raise BcsSerializationError(f"Unsupported MoveAction type: {action_arg.action}")
    
    @staticmethod
    def _encode_function_call(ser: BcsSerializer, func_arg: FunctionArgument):
        """Encode a FunctionArgument using the provided serializer.
        
        Args:
            ser: BcsSerializer instance
            func_arg: FunctionArgument object
            
        Raises:
            BcsSerializationError: If serialization fails
        """
        if not isinstance(func_arg, FunctionArgument):
            raise TypeError("Expected FunctionArgument object")

        # Sequence: function_id, ty_args, args
        
        # 1. function_id (FunctionId - assuming it becomes Serializable)
        if not isinstance(func_arg.function_id, FunctionId):
            raise TypeError("func_arg.function_id must be a FunctionId object")
        ser.struct(func_arg.function_id) # Assumes FunctionId implements serialize
        
        # 2. ty_args (Vec<TypeTag> - assuming TypeTag becomes Serializable)
        if not isinstance(func_arg.ty_args, list) or not all(isinstance(t, TypeTag) for t in func_arg.ty_args):
            raise TypeError("func_arg.ty_args must be a list of TypeTag objects")
        # Use ser.sequence with BcsSerializer.struct for Vec<Serializable>
        ser.sequence(func_arg.ty_args, BcsSerializer.struct) # Assumes TypeTag implements serialize
        
        # 3. args (Vec<Vec<u8>>) - BCS encode each raw value into its own Vec<u8>
        encoded_args_list: List[bytes] = []
        if not isinstance(func_arg.args, list):
            raise TypeError("func_arg.args must be a list")

        try:
            for raw_arg_value in func_arg.args:
                # Create a temporary serializer for each argument
                arg_ser = BcsSerializer()
                if isinstance(raw_arg_value, bool):
                    arg_ser.bool(raw_arg_value)
                elif isinstance(raw_arg_value, int):
                    # Assuming large integers should be u256 based on previous code?
                    # Or should this depend on the function signature?
                    # For now, sticking with u256 assumption.
                    arg_ser.u256(raw_arg_value)
                elif isinstance(raw_arg_value, str):
                    # Handle addresses vs other strings
                    if raw_arg_value.startswith("0x"):
                        try:
                            # Try parsing as RoochAddress
                            addr = RoochAddress.from_hex(raw_arg_value)
                            # Serialize RoochAddress (assuming it becomes Serializable)
                            arg_ser.struct(addr)
                        except ValueError:
                            # If not a valid address, treat as a plain string
                            arg_ser.str(raw_arg_value)
                    else:
                        arg_ser.str(raw_arg_value)
                elif isinstance(raw_arg_value, bytes):
                    # Serialize as variable-length bytes (Vec<u8>)
                    arg_ser.bytes(raw_arg_value)
                elif isinstance(raw_arg_value, Serializable):
                    # If the argument itself knows how to serialize, use that
                    arg_ser.struct(raw_arg_value)
                else:
                    raise BcsSerializationError(f"Unsupported raw argument type for BCS encoding: {type(raw_arg_value)}")
                encoded_args_list.append(arg_ser.output())

            # Serialize the list of already-serialized argument bytes
            # Each item in encoded_args_list is Vec<u8>, so use BcsSerializer.bytes
            ser.sequence(encoded_args_list, BcsSerializer.bytes)

        except Exception as e:
            raise BcsSerializationError(f"Failed to serialize function call args: {e}") from e
    
    @staticmethod
    def _encode_authenticator(auth: TransactionAuthenticator) -> bytes:
        """Encode a TransactionAuthenticator using the new BcsSerializer.
        
        Args:
            auth: TransactionAuthenticator object
            
        Returns:
            Encoded bytes
            
        Raises:
            BcsSerializationError: If serialization fails
        """
        if not isinstance(auth, TransactionAuthenticator):
            raise TypeError("Expected TransactionAuthenticator object")

        ser = BcsSerializer()
        try:
            # Sequence: auth_validator_id, payload
            
            # 1. auth_validator_id (u64)
            auth_id_map = {
                AuthenticatorType.ED25519: 0,
                AuthenticatorType.SECP256K1: 1,
                AuthenticatorType.SECP256R1: 2,
            }
            auth_validator_id = auth_id_map.get(auth.auth_type)
            if auth_validator_id is None:
                raise BcsSerializationError(f"Unsupported authenticator type for ID mapping: {auth.auth_type}")
            ser.u64(auth_validator_id)
            
            # 2. payload (Vec<u8>)
            # Payload structure depends on auth_validator_id
            if auth_validator_id == 0: # Ed25519 (Session)
                # Payload is Vec<u8> containing only the signature bytes
                if isinstance(auth.signature, str):
                    signature_bytes = from_hex(auth.signature)
                else:
                    signature_bytes = auth.signature
                ser.bytes(signature_bytes)
            elif auth_validator_id == 1: # Secp256k1 (Bitcoin)
                # Payload is the BCS encoding of the AuthPayload struct
                # Reconstruct AuthPayload data - THIS STILL REQUIRES FIXING
                # FIXME: How to get tx_hash or SignData here?
                # Using placeholders as before.

                if isinstance(auth.public_key, str):
                    public_key_bytes = from_hex(auth.public_key)
                else:
                    public_key_bytes = auth.public_key
                if isinstance(auth.signature, str):
                    signature_bytes = from_hex(auth.signature)
                else:
                    signature_bytes = auth.signature

                MESSAGE_PREFIX = b"Bitcoin Signed Message:\n"
                MESSAGE_INFO = b"Rooch Transaction:\n"
                # Placeholder for tx_hash_hex (usually 64 hex chars -> 32 bytes)
                # This needs to come from the actual transaction signing process
                tx_hash_bytes_placeholder = b'\x00' * 32 # 32 zero bytes placeholder

                message_info_full = MESSAGE_INFO + tx_hash_bytes_placeholder
                message_length = len(message_info_full)
                # Use the helper for varint, it should return bytes
                message_length_bytes = varint_byte_num(message_length)
                if not isinstance(message_length_bytes, bytes):
                    # Assuming varint_byte_num was modified to return bytes
                    # If it still returns int, need to encode it (e.g., with uleb128?)
                    # Let's assume it returns bytes for now.
                    pass
                message_prefix_full = MESSAGE_PREFIX + message_length_bytes

                # Placeholder Bitcoin address - derivation needed
                bitcoin_address_placeholder = f"bc1p_{auth.account_addr[:10]}..." # More distinct placeholder

                auth_payload_obj = AuthPayload(
                    signature=signature_bytes,
                    message_prefix=message_prefix_full,
                    message_info=message_info_full,
                    public_key=public_key_bytes,
                    from_address=bitcoin_address_placeholder # This needs real derivation
                )

                # Serialize the AuthPayload struct (assuming it becomes Serializable)
                ser.struct(auth_payload_obj)
            elif auth_validator_id == 2: # Secp256r1 (WebAuthn? Passkey?)
                # Payload is Vec<u8> containing only the signature bytes
                # Similar to Ed25519 for now, might need adjustment based on standard
                if isinstance(auth.signature, str):
                    signature_bytes = from_hex(auth.signature)
                else:
                    signature_bytes = auth.signature
                ser.bytes(signature_bytes)
            else:
                raise BcsSerializationError(f"Payload construction for auth_validator_id {auth_validator_id} not implemented.")

            return ser.output()
        except Exception as e:
            raise BcsSerializationError(f"Failed to serialize authenticator: {e}. Authenticator: {auth}") from e

    @staticmethod
    def encode_signed_transaction(signed_tx: SignedTransaction) -> bytes:
        """Encode a signed transaction by combining encoded data and authenticator.
        
        Args:
            signed_tx: Signed transaction object
            
        Returns:
            Encoded transaction bytes
            
        Raises:
            BcsSerializationError: If serialization fails
        """
        if not isinstance(signed_tx, SignedTransaction):
            raise TypeError("Expected SignedTransaction object")
        try:
            # Sequence: data, authenticator
            tx_data_bytes = TxSerializer.encode_transaction_data(signed_tx.tx_data)
            auth_bytes = TxSerializer._encode_authenticator(signed_tx.authenticator)
            return tx_data_bytes + auth_bytes
        except Exception as e:
            raise BcsSerializationError(f"Failed to serialize signed transaction: {e}") from e
    
    @staticmethod
    def encode_transaction_for_submission(signed_tx: SignedTransaction) -> str:
        """Encode a signed transaction as hex for submission.
        
        Args:
            signed_tx: Signed transaction
            
        Returns:
            Hex-encoded transaction data
            
        Raises:
            BcsSerializationError: If serialization fails
        """
        encoded_bytes = TxSerializer.encode_signed_transaction(signed_tx)
        return to_hex(encoded_bytes)