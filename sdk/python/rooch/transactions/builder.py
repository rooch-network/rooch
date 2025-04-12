#!/usr/bin/env python3
# Copyright (c) Rooch Network
# SPDX-License-Identifier: Apache-2.0

import time
from typing import Any, Dict, List, Optional, Union

from ..address.rooch import RoochAddress
from ..bcs.serializer import Args
from ..crypto.signer import Signer
from .serializer import TxSerializer
from .types import (
    AuthenticatorType,
    FunctionArgument,
    MoveAction,
    MoveActionArgument,
    SignedTransaction,
    TransactionArgument,
    TransactionAuthenticator,
    TransactionData,
    TransactionType,
)


class TransactionBuilder:
    """Builder for Rooch transactions"""
    
    def __init__(
        self,
        sender_address: str,
        sequence_number: int,
        max_gas_amount: int = 10_000_000,
        gas_unit_price: int = 1,
        expiration_timestamp_secs: int = 0,
        chain_id: int = 1
    ):
        """Initialize a transaction builder
        
        Args:
            sender_address: Sender account address
            sequence_number: Transaction sequence number
            max_gas_amount: Maximum gas amount
            gas_unit_price: Gas unit price
            expiration_timestamp_secs: Expiration timestamp in seconds
            chain_id: Chain ID
        """
        self.sender_address = sender_address
        self.sequence_number = sequence_number
        self.max_gas_amount = max_gas_amount
        self.gas_unit_price = gas_unit_price
        self.expiration_timestamp_secs = expiration_timestamp_secs
        self.chain_id = chain_id
    
    def build_function_payload(
        self,
        function_id: str,
        ty_args: Optional[List[str]] = None,
        args: Optional[List[List[Any]]] = None
    ) -> MoveActionArgument:
        """Build a function payload
        
        Args:
            function_id: Function ID (module::function)
            ty_args: Optional type arguments
            args: Optional function arguments (created using Args)
            
        Returns:
            MoveActionArgument for function call
        """
        if ty_args is None:
            ty_args = []
            
        if args is None:
            args = []
            
        tx_args = [TransactionArgument(arg[0], arg[1]) for arg in args]
        function_arg = FunctionArgument(function_id, ty_args, tx_args)
        
        return MoveActionArgument(MoveAction.FUNCTION, function_arg)
    
    def build_script_payload(self, script: str) -> MoveActionArgument:
        """Build a script payload
        
        Args:
            script: Move script
            
        Returns:
            MoveActionArgument for script execution
        """
        return MoveActionArgument(MoveAction.SCRIPT, script)
    
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
            chain_id=self.chain_id
        )
    
    def build_module_publish_tx(self, module_bytes: Union[bytes, str]) -> TransactionData:
        """Build a module publish transaction
        
        Args:
            module_bytes: Move module bytecode
            
        Returns:
            TransactionData for the module publish
        """
        return TransactionData(
            tx_type=TransactionType.MOVE_MODULE_TRANSACTION,
            tx_arg=module_bytes,
            sequence_number=self.sequence_number,
            max_gas_amount=self.max_gas_amount,
            gas_unit_price=self.gas_unit_price,
            expiration_timestamp_secs=self.expiration_timestamp_secs,
            chain_id=self.chain_id
        )
    
    def sign(self, tx_data: TransactionData, signer: Signer) -> SignedTransaction:
        """Sign a transaction
        
        Args:
            tx_data: Transaction data
            signer: Signer for the transaction
            
        Returns:
            Signed transaction ready for submission
        """
        # Encode the transaction data for signing
        serialized_tx = TxSerializer.encode_transaction_data(tx_data)
        
        # Sign the transaction
        signature = signer.sign(serialized_tx)
        
        # Create the authenticator
        auth = TransactionAuthenticator(
            account_addr=self.sender_address,
            public_key=signer.get_public_key(),
            signature=signature,
            auth_type=AuthenticatorType.ED25519  # Assuming ED25519 for now
        )
        
        return SignedTransaction(tx_data, auth)
    
    @staticmethod
    def with_default_account(
        signer: Signer,
        sequence_number: int,
        chain_id: int = 1,
        max_gas_amount: int = 10_000_000,
        gas_unit_price: int = 1,
        expiration_delta_secs: int = 600  # 10 minutes
    ) -> 'TransactionBuilder':
        """Create a TransactionBuilder with defaults from a signer
        
        Args:
            signer: Signer for the transaction
            sequence_number: Transaction sequence number
            chain_id: Chain ID
            max_gas_amount: Maximum gas amount
            gas_unit_price: Gas unit price
            expiration_delta_secs: Expiration time delta in seconds
            
        Returns:
            TransactionBuilder configured with the signer's address
        """
        expiration_timestamp_secs = int(time.time()) + expiration_delta_secs
        
        return TransactionBuilder(
            sender_address=signer.get_address(),
            sequence_number=sequence_number,
            max_gas_amount=max_gas_amount,
            gas_unit_price=gas_unit_price,
            expiration_timestamp_secs=expiration_timestamp_secs,
            chain_id=chain_id
        )