#!/usr/bin/env python3
# Copyright (c) Rooch Network
# SPDX-License-Identifier: Apache-2.0

import asyncio
from typing import Any, Dict, List, Optional, Union

import aiohttp

from ..transport import RoochEnvironment, RoochTransport, RoochTransportError
from .account import AccountClient
from .transaction import TransactionClient
from ..crypto.signer import Signer
from ..transactions.builder import TransactionBuilder
from ..transactions.types import (
    AuthenticatorType,
    FunctionArgument,
    MoveAction,
    MoveActionArgument,
    SignedTransaction,
    TransactionData,
    TransactionType,
    ModuleId,
    FunctionId,
    TypeTag,
)


class RoochClient:
    """Main Rooch client"""
    
    def __init__(
        self,
        url_or_env: str = RoochEnvironment.LOCAL,
        request_timeout_ms: int = 30000,
        session: Optional[aiohttp.ClientSession] = None,
        headers: Optional[Dict[str, str]] = None
    ):
        """Initialize with endpoint URL or environment
        
        Args:
            url_or_env: URL of the Rooch RPC endpoint or predefined environment
            request_timeout_ms: Request timeout in milliseconds
            session: Optional aiohttp client session
            headers: Optional additional HTTP headers
        """
        # Create a transport
        self._transport = RoochTransport(
            url=url_or_env,
            request_timeout_ms=request_timeout_ms,
            session=session,
            headers=headers
        )
        
        # Create clients
        self.account = AccountClient(self._transport)
        self.transaction = TransactionClient(self._transport)
    
    async def get_chain_id(self) -> int:
        """Get chain ID
        
        Returns:
            Chain ID
        """
        # Use the correct RPC method name from openrpc.json
        result = await self._transport.request("rooch_getChainID")
        # The result seems to be a string representation of u64
        return int(result)
    
    async def get_states(self, cursor: int = 0, limit: int = 25) -> Dict[str, Any]:
        """Get global states with pagination
        
        Args:
            cursor: Starting cursor
            limit: Maximum number of states to return
            
        Returns:
            List of states and pagination info
        """
        return await self._transport.request("rooch_getStates", [cursor, limit])
    
    async def get_state_by_state_key(self, state_key: str) -> Dict[str, Any]:
        """Get state by state key
        
        Args:
            state_key: State key
            
        Returns:
            State information
        """
        return await self._transport.request("rooch_getStateByStateKey", [state_key])
    
    async def get_states_by_prefix(
        self, 
        prefix: str, 
        cursor: int = 0, 
        limit: int = 25
    ) -> Dict[str, Any]:
        """Get states by prefix with pagination
        
        Args:
            prefix: Key prefix
            cursor: Starting cursor
            limit: Maximum number of states to return
            
        Returns:
            List of states and pagination info
        """
        return await self._transport.request("rooch_getStatesByPrefix", [prefix, cursor, limit])
    
    async def get_current_epoch(self) -> int:
        """Get current epoch
        
        Returns:
            Current epoch number
        """
        return await self._transport.request("rooch_getCurrentEpoch")
    
    async def get_block_by_height(self, height: int) -> Dict[str, Any]:
        """Get block by height
        
        Args:
            height: Block height
            
        Returns:
            Block information
        """
        return await self._transport.request("rooch_getBlockByHeight", [height])
    
    async def get_block_info_by_height(self, height: int) -> Dict[str, Any]:
        """Get block info by height
        
        Args:
            height: Block height
            
        Returns:
            Block info
        """
        return await self._transport.request("rooch_getBlockInfoByHeight", [height])
    
    async def get_transaction_builder(
        self, 
        sender_address: str,
        signer: Optional[Signer] = None,
        max_gas_amount: int = 10_000_000,
        gas_unit_price: int = 1,
        expiration_delta_secs: int = 600  # 10 minutes
    ) -> TransactionBuilder:
        """Get a transaction builder for the sender
        
        Args:
            sender_address: Sender account address
            signer: Optional signer (if provided, will use the address from it)
            max_gas_amount: Maximum gas amount
            gas_unit_price: Gas unit price
            expiration_delta_secs: Expiration time delta in seconds
            
        Returns:
            Configured TransactionBuilder
        """
        # If signer is provided, use its address
        if signer is not None:
            sender_address = signer.get_address()
        
        # Get account sequence number
        sequence_number = await self.account.get_account_sequence_number(sender_address)
        
        # Get chain ID
        chain_id = await self.get_chain_id()
        
        # Create a TransactionBuilder
        return TransactionBuilder(
            sender_address=sender_address,
            sequence_number=sequence_number,
            chain_id=chain_id,
            max_gas_amount=max_gas_amount
        )
    
    async def submit_and_wait(
        self, 
        transaction: SignedTransaction,
        timeout_secs: int = 60,
        poll_interval_ms: int = 1000
    ) -> Dict[str, Any]:
        """Submit a transaction and wait for confirmation
        
        Args:
            transaction: Signed transaction
            timeout_secs: Wait timeout in seconds
            poll_interval_ms: Polling interval in milliseconds
            
        Returns:
            Transaction information when confirmed
        """
        # Submit transaction
        tx_hash = await self.transaction.submit_transaction(transaction)
        
        # Wait for confirmation
        return await self.transaction.wait_for_transaction(
            tx_hash,
            timeout_secs=timeout_secs,
            poll_interval_ms=poll_interval_ms
        )
    
    async def _prepare_move_call_tx_data(
        self,
        signer: Signer,
        function_id: str,
        type_args: List[TypeTag],
        args: List[Any],
        max_gas_amount: Optional[int] = None,
    ) -> TransactionData:
        """Helper to prepare TransactionData for a Move function call."""
        # ... (get sender_addr, seq_num, chain_id, gas) ...
        
        # Parse function_id string into FunctionId object
        try:
            parts = function_id.split("::")
            if len(parts) != 3:
                 raise ValueError("Invalid function_id format. Expected 'address::module::function'")
            addr_str, module_name, func_name = parts
            # Basic validation for short addresses before creating ModuleId
            if addr_str.startswith('0x') and len(addr_str) < 66:
                try:
                    int(addr_str[2:], 16)
                except ValueError:
                    raise ValueError(f"Invalid short hex address: {addr_str}")
            # Add more robust validation if needed
            
            mod_id = ModuleId(address=addr_str, name=module_name)
            func_id_obj = FunctionId(module_id=mod_id, function_name=func_name)
        except Exception as e:
            raise ValueError(f"Failed to parse function_id '{function_id}': {e}") from e
            
        # Validate type_args are TypeTag objects
        if not all(isinstance(arg, TypeTag) for arg in type_args):
            raise TypeError("type_args must be a list of TypeTag objects")

        action_args = FunctionArgument(
            function_id=func_id_obj,
            ty_args=type_args,
            args=args
        )
        action = MoveActionArgument(MoveAction.FUNCTION, action_args)

        tx_data = TransactionData(
            sender=sender_addr,
            sequence_number=seq_num,
            chain_id=self.chain_id,
            max_gas_amount=gas,
            action=action
        )
        return tx_data
        
    async def execute_move_call(
        self,
        signer: Signer,
        function_id: str,
        type_args: Optional[List[str]] = None,
        args: Optional[List[List[Any]]] = None,
        max_gas_amount: int = 10_000_000,
        gas_unit_price: int = 1,
        expiration_delta_secs: int = 600
    ) -> Dict[str, Any]:
        """Execute a Move function call
        
        Args:
            signer: Transaction signer
            function_id: Function ID (module::function)
            type_args: Type arguments
            args: Function arguments
            max_gas_amount: Maximum gas amount
            gas_unit_price: Gas unit price
            expiration_delta_secs: Expiration time delta in seconds
            
        Returns:
            Transaction execution result
        """
        # Create a transaction builder
        tx_builder = await self.get_transaction_builder(
            sender_address=signer.get_address(),
            signer=None,  # We already have the address
            max_gas_amount=max_gas_amount,
            gas_unit_price=gas_unit_price,
            expiration_delta_secs=expiration_delta_secs
        )
        
        # Build function payload
        payload = tx_builder.build_function_payload(
            function_id=function_id,
            ty_args=type_args,
            args=args
        )
        
        # Build transaction
        tx_data = tx_builder.build_move_action_tx(payload)
        
        # Sign transaction
        signed_tx = tx_builder.sign(tx_data, signer)
        
        # Submit and wait for confirmation
        return await self.submit_and_wait(signed_tx)
    
    async def publish_module(
        self,
        signer: Signer,
        module_bytes: Union[bytes, str],
        max_gas_amount: int = 10_000_000,
        gas_unit_price: int = 1,
        expiration_delta_secs: int = 600
    ) -> Dict[str, Any]:
        """Publish a Move module
        
        Args:
            signer: Transaction signer
            module_bytes: Module bytecode (bytes or hex string)
            max_gas_amount: Maximum gas amount
            gas_unit_price: Gas unit price
            expiration_delta_secs: Expiration time delta in seconds
            
        Returns:
            Transaction execution result
        """
        # Create a transaction builder
        tx_builder = await self.get_transaction_builder(
            sender_address=signer.get_address(),
            signer=None,  # We already have the address
            max_gas_amount=max_gas_amount,
            gas_unit_price=gas_unit_price,
            expiration_delta_secs=expiration_delta_secs
        )
        
        # Build transaction
        tx_data = tx_builder.build_module_publish_tx(module_bytes)
        
        # Sign transaction
        signed_tx = tx_builder.sign(tx_data, signer)
        
        # Submit and wait for confirmation
        return await self.submit_and_wait(signed_tx)
    
    async def close(self):
        """Close the underlying transport session if it was created by the client."""
        # Access the underlying transport session
        session = self._transport.session
        should_close = self._transport._should_close_session # Assuming transport tracks this

        if session and should_close:
            await session.close()
    
    async def __aenter__(self):
        """Enter context manager"""
        # The transport is initialized in __init__.
        # No specific action needed here unless we defer session creation.
        return self
    
    async def __aexit__(self, exc_type, exc_val, exc_tb):
        """Exit context manager and close session"""
        await self.close()