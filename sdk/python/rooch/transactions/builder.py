#!/usr/bin/env python3
# Copyright (c) Rooch Network
# SPDX-License-Identifier: Apache-2.0

import time
from typing import Any, Dict, List, Optional, Union
import hashlib

from ..address.rooch import RoochAddress
from ..crypto.signer import Signer
from ..crypto.signer import RoochSigner
from ..utils.hex import from_hex
from .serializer import TxSerializer
from .types import (
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


class TransactionBuilder:
    """Builder for Rooch transactions"""
    
    def __init__(
        self,
        sender_address: str,
        sequence_number: int,
        chain_id: int,
        max_gas_amount: int = 10_000_000,
    ):
        """Initialize a transaction builder based on new TransactionData fields
        
        Args:
            sender_address: Sender account address
            sequence_number: Transaction sequence number
            chain_id: Chain ID
            max_gas_amount: Maximum gas amount
        """
        self.sender_address = sender_address
        self.sequence_number = sequence_number
        self.chain_id = chain_id
        self.max_gas_amount = max_gas_amount
    
    def build_function_payload(
        self,
        function_id: str,
        ty_args: Optional[List[Union[str, TypeTag]]] = None,
        args: Optional[List[Any]] = None
    ) -> MoveActionArgument:
        """Build a function payload for MoveAction
        
        Args:
            function_id: Function ID (module::function)
            ty_args: Optional type arguments (can be strings or TypeTag objects)
            args: Optional list of raw argument values
            
        Returns:
            MoveActionArgument for function call
        """
        ty_args = ty_args or []
        args = args or []
        
        # Convert string type args to TypeTag objects
        converted_ty_args = []
        for ty_arg in ty_args:
            if isinstance(ty_arg, str):
                # Parse string like "0x3::gas_coin::RGas"
                parts = ty_arg.split("::")
                if len(parts) != 3:
                    raise ValueError(f"Invalid type argument format: {ty_arg}")
                addr_str, module, name = parts
                struct_tag = StructTag(address=addr_str, module=module, name=name, type_params=[])
                converted_ty_args.append(TypeTag.struct(struct_tag))
            elif isinstance(ty_arg, TypeTag):
                converted_ty_args.append(ty_arg)
            else:
                raise TypeError(f"Invalid type argument: {ty_arg}")
        
        try:
            parts = function_id.split("::")
            if len(parts) != 3:
                 raise ValueError("Invalid function_id format. Expected 'address::module::function'")
            addr_str, module_name, func_name = parts
            if addr_str.startswith('0x') and len(addr_str) < 66:
                try:
                    int(addr_str[2:], 16)
                except ValueError:
                    raise ValueError(f"Invalid short hex address: {addr_str}")
            
            mod_id = ModuleId(address=addr_str, name=module_name)
            func_id_obj = FunctionId(module_id=mod_id, function_name=func_name)
        except Exception as e:
            raise ValueError(f"Failed to parse function_id '{function_id}': {e}") from e
            
        func_arg = FunctionArgument(function_id=func_id_obj, ty_args=converted_ty_args, args=args)
        
        return MoveActionArgument(MoveAction.FUNCTION, func_arg)
    
    def build_move_action_tx(self, action_arg: MoveActionArgument) -> TransactionData:
        """Build a Move action transaction
        
        Args:
            action_arg: Move action argument
            
        Returns:
            TransactionData for the action
        """
        return TransactionData(
            tx_type=TransactionType.MOVE_ACTION,
            tx_arg=action_arg,
            sequence_number=self.sequence_number,
            max_gas_amount=self.max_gas_amount,
            chain_id=self.chain_id,
            sender=RoochAddress.from_hex(self.sender_address)
        )
    
    def build_module_publish_tx(self, module_bytes: Union[bytes, str]) -> TransactionData:
        """Build a module publish transaction
        
        Args:
            module_bytes: Move module bytecode (bytes or hex string)
            
        Returns:
            TransactionData for the module publish
        """
        if isinstance(module_bytes, str):
            module_bytes = from_hex(module_bytes)
        elif not isinstance(module_bytes, bytes):
            raise TypeError("module_bytes must be bytes or hex string")

        return TransactionData(
            tx_type=TransactionType.MOVE_MODULE_TRANSACTION,
            tx_arg=module_bytes,
            sequence_number=self.sequence_number,
            max_gas_amount=self.max_gas_amount,
            chain_id=self.chain_id,
            sender=RoochAddress.from_hex(self.sender_address)
        )
    
    def sign(self, tx_data: TransactionData, signer: Signer) -> SignedTransaction:
        """Sign transaction data and create a signed transaction"""
        # 1. Serialize TransactionData
        serialized_tx_data = TxSerializer.encode_transaction_data(tx_data)

        # 2. Calculate SHA3-256 hash of serialized data
        tx_hash_bytes = hashlib.sha3_256(serialized_tx_data).digest()
        
        # 3. Sign the hash using KeyPair's sign_digest method
        # Access the KeyPair from the Signer (assuming RoochSigner)
        if not isinstance(signer, RoochSigner):
             # We might need a more generic way if other Signer types exist
             raise TypeError("Expected RoochSigner to access KeyPair")
        keypair = signer.get_keypair()

        # In Bitcoin signing, the message is first prefixed and then hashed (twice).
        # The prefix is "Bitcoin Signed Message:" + length_of_message
        prefix = b"Bitcoin Signed Message:\n"
        message_len = len(tx_hash_bytes)
        
        # The length is encoded as a single byte for messages up to 252 bytes.
        # ref: https://en.bitcoin.it/wiki/Protocol_documentation#Variable_length_integer
        if message_len <= 252:
            len_bytes = bytes([message_len])
        else:
            # Note: This part is simplified. For larger messages, a multi-byte varint is needed.
            # However, transaction hashes are fixed-size and small, so this is sufficient.
            raise ValueError("Message length exceeds 252 bytes, which is not supported by this simplified varint encoding.")

        full_message = prefix + len_bytes + tx_hash_bytes
        
        # Double SHA256 hash
        digest = hashlib.sha256(full_message).digest()
        digest = hashlib.sha256(digest).digest()

        # Sign the final hash
        signature_bytes = keypair.sign(digest)

        # 4. Create Bitcoin authenticator instead of Session authenticator
        # Bitcoin authenticator uses secp256k1 keys and is the primary authentication method
        # Session authenticator requires pre-created session keys
        
        # Get public key and generate Bitcoin address
        public_key_bytes = keypair.get_public_key()
        from ..address.bitcoin import BitcoinAddress
        bitcoin_address = BitcoinAddress.from_public_key(public_key_bytes, mainnet=True)
        bitcoin_address_str = str(bitcoin_address)
        
        # Create AuthPayload structure for Bitcoin authenticator
        # Based on Rust AuthPayload::new implementation
        from ..bcs.serializer import BcsSerializer
        auth_payload_serializer = BcsSerializer()
        
        # Serialize AuthPayload fields: signature, message_prefix, message_info, public_key, from_address
        auth_payload_serializer.bytes(signature_bytes)  # signature
        auth_payload_serializer.bytes(b"Bitcoin Signed Message:\n")  # message_prefix (default)
        auth_payload_serializer.bytes(b"Rooch Transaction:\n")  # message_info (default, without tx hash)
        auth_payload_serializer.bytes(public_key_bytes)  # public_key
        auth_payload_serializer.bytes(bitcoin_address_str.encode('utf-8'))  # from_address
        
        auth_payload_bytes = auth_payload_serializer.output()
        auth = TransactionAuthenticator.bitcoin(auth_payload_bytes)
        # 6. Return SignedTransaction
        return SignedTransaction(tx_data, auth)

    def build_signed_tx(
        self,
        payload: MoveActionArgument,
        signer: Signer
    ) -> SignedTransaction:
        # ... existing logic ...
        tx_data = self.build_move_action_tx(payload)
        return self.sign(tx_data, signer)

    @classmethod
    def with_default_account(
        cls,
        signer: Signer,
        sequence_number: int,
        chain_id: int,
        max_gas_amount: int = 10_000_000,
    ) -> 'TransactionBuilder':
        """Create a transaction builder with default account settings
        
        Args:
            signer: Transaction signer
            sequence_number: Transaction sequence number
            chain_id: Chain ID
            max_gas_amount: Maximum gas amount
            
        Returns:
            TransactionBuilder instance
        """
        return cls(
            sender_address=signer.get_address(),
            sequence_number=sequence_number,
            chain_id=chain_id,
            max_gas_amount=max_gas_amount,
        )