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
from ..session.session import Session, CreateSessionArgs


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
        result = await self._transport.request("rooch_getChainID")
        return int(result)

    async def get_states(self, access_path: str, state_option: Optional[dict] = None) -> List[Optional[dict]]:
        """Get the states by access_path (see Rust RoochAPI)
        Args:
            access_path: The access path string (required)
            state_option: Optional state options dict
        Returns:
            List of state objects or None
        """
        if not access_path:
            raise ValueError("access_path is required for get_states (see RoochAPI)")
        params = [access_path]
        if state_option is not None:
            params.append(state_option)
        return await self._transport.request("rooch_getStates", params)

    async def list_states(self, access_path: str, cursor: Optional[str] = None, limit: Optional[int] = None, state_option: Optional[dict] = None) -> dict:
        """List the states by access_path (see Rust RoochAPI)
        Args:
            access_path: The access path string
            cursor: Optional cursor string
            limit: Optional limit (int)
            state_option: Optional state options dict
        Returns:
            State page dict
        """
        params = [access_path, cursor, limit, state_option]
        return await self._transport.request("rooch_listStates", params)

    async def get_block_by_height(self, height: int) -> dict:
        """Get block by height (Rust: getBlockByHeight)"""
        return await self._transport.request("rooch_getBlockByHeight", [height])

    async def get_transaction_builder(
        self, 
        sender_address: str,
        signer: Optional[Signer] = None,
        max_gas_amount: int = 10_000_000,
    ) -> TransactionBuilder:
        """Get a transaction builder for the sender
        
        Args:
            sender_address: Sender account address
            signer: Optional signer (if provided, will use the address from it)
            max_gas_amount: Maximum gas amount
            
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
            max_gas_amount=max_gas_amount,
        )
    
    async def submit_and_wait(
        self, 
        transaction: SignedTransaction,
        timeout_secs: int = 60,
        poll_interval_ms: int = 1000
    ) -> Dict[str, Any]:
        """Submit a transaction and get the execution result
        
        Args:
            transaction: Signed transaction
            timeout_secs: Wait timeout in seconds (not used, kept for backward compatibility)
            poll_interval_ms: Polling interval in milliseconds (not used, kept for backward compatibility)
            
        Returns:
            Transaction execution result
        """
        # Submit transaction and get immediate execution result
        return await self.transaction.submit_transaction(transaction)
    
    
        
    async def execute_move_call(
        self,
        signer: Signer,
        function_id: str,
        type_args: Optional[List[str]] = None,
        args: Optional[List[List[Any]]] = None,
        max_gas_amount: int = 10_000_000,
    ) -> Dict[str, Any]:
        """Execute a Move function call (Rust: executeRawTransaction, but Python builds and signs tx)
        
        Args:
            signer: Transaction signer
            function_id: Function ID (module::function)
            type_args: Type arguments
            args: Function arguments
            max_gas_amount: Maximum gas amount
            
        Returns:
            Transaction execution result
        """
        # Create a transaction builder
        tx_builder = await self.get_transaction_builder(
            sender_address=signer.get_address(),
            max_gas_amount=max_gas_amount,
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
        
        # Submit transaction and get immediate execution result
        return await self.transaction.submit_transaction(signed_tx)
    
    async def publish_module(
        self,
        signer: Signer,
        module_bytes: Union[bytes, str],
        max_gas_amount: int = 10_000_000,
    ) -> dict:
        """Publish a Move module (Rust: executeRawTransaction with module publish payload)
        
        Args:
            signer: Transaction signer
            module_bytes: Module bytecode (bytes or hex string)
            max_gas_amount: Maximum gas amount

        Returns:
            Transaction execution result
        """
        # Create a transaction builder
        tx_builder = await self.get_transaction_builder(
            sender_address=signer.get_address(),
            max_gas_amount=max_gas_amount,
        )
        
        # Build transaction
        tx_data = tx_builder.build_module_publish_tx(module_bytes)
        
        # Sign transaction
        signed_tx = tx_builder.sign(tx_data, signer)
        
        # Submit and wait for confirmation
        return await self.submit_and_wait(signed_tx)

    async def create_session(self, session_args: CreateSessionArgs, signer: Signer) -> Session:
        """Create a new session key (Rust: session key logic is on-chain, Python builds tx and calls)

        Args:
            session_args: Arguments for creating the session.
            signer: The signer to authorize the session creation.

        Returns:
            A Session object representing the newly created session.
        """
        return await Session.create(client=self, signer=signer, session_args=session_args)
    
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