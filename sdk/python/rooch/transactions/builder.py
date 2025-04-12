#!/usr/bin/env python3
# Copyright (c) Rooch Network
# SPDX-License-Identifier: Apache-2.0

import time
from typing import Any, Dict, List, Optional, Union
import hashlib

from ..address.rooch import RoochAddress
from ..bcs.serializer import Args
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
    TypeTag
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
        ty_args: Optional[List[TypeTag]] = None,
        args: Optional[List[Any]] = None
    ) -> MoveActionArgument:
        """Build a function payload for MoveAction
        
        Args:
            function_id: Function ID (module::function)
            ty_args: Optional type arguments
            args: Optional list of raw argument values
            
        Returns:
            MoveActionArgument for function call
        """
        ty_args = ty_args or []
        args = args or []
        
        if not all(isinstance(arg, TypeTag) for arg in ty_args):
            raise TypeError("ty_args must be a list of TypeTag objects")

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
            
        func_arg = FunctionArgument(function_id=func_id_obj, ty_args=ty_args, args=args)
        
        return MoveActionArgument(MoveAction.FUNCTION, func_arg)
    
    def build_move_action_tx(self, action_arg: MoveActionArgument) -> TransactionData:
        """Build a Move action transaction
        
        Args:
            action_arg: Move action argument
            
        Returns:
            TransactionData for the action
        """
        return TransactionData(
            sender=self.sender_address,
            sequence_number=self.sequence_number,
            chain_id=self.chain_id,
            max_gas_amount=self.max_gas_amount,
            action=action_arg,
        )
    
    def build_module_publish_tx(self, module_bytes: Union[bytes, str]) -> TransactionData:
        """Build a module publish transaction
        
        Args:
            module_bytes: Move module bytecode (list of bytes)
            
        Returns:
            TransactionData for the module publish
        """
        if isinstance(module_bytes, str):
            module_bytes_list = [from_hex(module_bytes)]
        elif isinstance(module_bytes, bytes):
             module_bytes_list = [module_bytes]
        elif isinstance(module_bytes, list):
            module_bytes_list = [
                from_hex(b) if isinstance(b, str) else b for b in module_bytes
            ]
        else:
            raise TypeError("module_bytes must be bytes, hex string, list of bytes, or list of hex strings")

        action_arg = MoveActionArgument(MoveAction.MODULE_BUNDLE, module_bytes_list)
        
        return TransactionData(
            sender=self.sender_address,
            sequence_number=self.sequence_number,
            chain_id=self.chain_id,
            max_gas_amount=self.max_gas_amount,
            action=action_arg,
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
        keypair = signer.get_keypair() # Assuming get_keypair() exists on RoochSigner
        signature_bytes = keypair.sign_digest(tx_hash_bytes) # Use sign_digest

        # 4. Determine AuthenticatorType based on KeyPair/Signer scheme
        # Assuming KeyPair uses P-256 which corresponds to SECP256R1
        # TODO: Add a get_scheme() method to KeyPair/Signer later if needed
        auth_type = AuthenticatorType.SECP256R1 

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