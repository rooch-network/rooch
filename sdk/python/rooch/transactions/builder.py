#!/usr/bin/env python3
# Copyright (c) Rooch Network
# SPDX-License-Identifier: Apache-2.0

import time
from typing import Any, Dict, List, Optional, Union
import hashlib

from ..address.rooch import RoochAddress
from ..crypto.signer import Signer
from ..crypto.signer import RoochSigner
from .serializer import TxSerializer
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


class TransactionBuilder:
    """Builder for Rooch transactions"""
    
    def __init__(
        self,
        sender_address: str,
        sequence_number: int,
        chain_id: int,
        max_gas_amount: int = 10_000_000,
        gas_unit_price: int = 1,
        expiration_timestamp_secs: int = 0,
    ):
        """Initialize a transaction builder based on new TransactionData fields
        
        Args:
            sender_address: Sender account address
            sequence_number: Transaction sequence number
            chain_id: Chain ID
            max_gas_amount: Maximum gas amount
            gas_unit_price: Gas unit price
            expiration_timestamp_secs: Expiration timestamp in seconds
        """
        self.sender_address = sender_address
        self.sequence_number = sequence_number
        self.chain_id = chain_id
        self.max_gas_amount = max_gas_amount
        self.gas_unit_price = gas_unit_price
        self.expiration_timestamp_secs = expiration_timestamp_secs
    
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
            gas_unit_price=self.gas_unit_price,
            expiration_timestamp_secs=self.expiration_timestamp_secs,
            chain_id=self.chain_id,
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
            gas_unit_price=self.gas_unit_price,
            expiration_timestamp_secs=self.expiration_timestamp_secs,
            chain_id=self.chain_id,
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
        signature_bytes = keypair.sign_digest(tx_hash_bytes)

        # 4. Determine AuthenticatorType based on KeyPair/Signer scheme
        # Assuming KeyPair now defaults to SECP256K1
        auth_type = AuthenticatorType.SECP256K1 

        # 5. Create TransactionAuthenticator
        auth = TransactionAuthenticator(
            account_addr=tx_data.sender, # Sender from TransactionData
            public_key=keypair.get_public_key(), # Get uncompressed pubkey
            signature=signature_bytes, # Use raw signature bytes
            auth_type=auth_type 
        )
        
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
        gas_unit_price: int = 1,
        expiration_delta_secs: int = 3600,
    ) -> 'TransactionBuilder':
        """Create a transaction builder with default account settings
        
        Args:
            signer: Transaction signer
            sequence_number: Transaction sequence number
            chain_id: Chain ID
            max_gas_amount: Maximum gas amount
            gas_unit_price: Gas unit price
            expiration_delta_secs: Expiration delta in seconds
            
        Returns:
            TransactionBuilder instance
        """
        expiration_timestamp_secs = int(time.time()) + expiration_delta_secs
        return cls(
            sender_address=signer.get_address(),
            sequence_number=sequence_number,
            chain_id=chain_id,
            max_gas_amount=max_gas_amount,
            gas_unit_price=gas_unit_price,
            expiration_timestamp_secs=expiration_timestamp_secs,
        )