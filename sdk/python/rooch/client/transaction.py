#!/usr/bin/env python3
# Copyright (c) Rooch Network
# SPDX-License-Identifier: Apache-2.0

from typing import Any, Dict, List, Optional, Union
import asyncio

from ..transport import RoochTransport
from ..transactions.serializer import TxSerializer
from ..transactions.types import SignedTransaction
from ..transport import RoochTransportError


class TransactionClient:
    """Client for Rooch transaction operations"""
    
    def __init__(self, transport: RoochTransport):
        """Initialize with a transport
        
        Args:
            transport: Transport for communicating with the Rooch node
        """
        self._transport = transport
    
    async def submit_transaction(self, transaction: SignedTransaction) -> str:
        """Submit a signed transaction using executeRawTransaction
        
        Args:
            transaction: Signed transaction
            
        Returns:
            Transaction hash (or execution result structure)
        """
        # Serialize the transaction for raw submission (usually hex string)
        tx_hex = TxSerializer.encode_transaction_for_submission(transaction)
        
        # Submit the transaction using the correct RPC method
        # Note: The return type might be different for executeRawTransaction (e.g., ExecuteTransactionResponseView)
        # Adjust return type annotation and handling if necessary based on actual RPC response.
        return await self._transport.request("rooch_executeRawTransaction", [tx_hex])
    
    async def get_transaction_by_hash(self, transaction_hash: str) -> Optional[Dict[str, Any]]:
        """Get transaction by hash
        
        Args:
            transaction_hash: Transaction hash
            
        Returns:
            Transaction information or None if not found
        """
        # Ensure hash has 0x prefix and is likely 32 bytes (64 hex chars + 2 for 0x)
        if not transaction_hash.startswith("0x"):
            transaction_hash = "0x" + transaction_hash
        # Basic length check, though server might have stricter validation
        if len(transaction_hash) != 66:
             print(f"Warning: Transaction hash {transaction_hash} length is not 66. Proceeding anyway.")
             # raise ValueError(f"Invalid H256 hash format: {transaction_hash}")

        # Use the correct RPC method name and expect a list in response
        results = await self._transport.request(
            "rooch_getTransactionsByHash",
            [transaction_hash] # Pass validated hash string in a list
        )
        # The result is a list of options, return the first element if it exists
        if results and len(results) > 0 and results[0] is not None:
            return results[0]
        else:
            return None
    
    async def get_transaction_info_by_hash(self, transaction_hash: str) -> Dict[str, Any]:
        """Get transaction info by hash
        
        Args:
            transaction_hash: Transaction hash
            
        Returns:
            Transaction info
        """
        # TODO: Verify if this method still exists or needs update based on openrpc.json
        return await self._transport.request("rooch_getTransactionInfoByHash", [transaction_hash])
    
    async def wait_for_transaction(
        self,
        transaction_hash: str,
        timeout_secs: int = 60,
        poll_interval_ms: int = 1000
    ) -> Dict[str, Any]:
        """Wait for transaction to be confirmed by polling.

        Args:
            transaction_hash: Transaction hash
            timeout_secs: Timeout in seconds
            poll_interval_ms: Polling interval in milliseconds

        Returns:
            Transaction information when confirmed

        Raises:
            TimeoutError: If the transaction is not confirmed within the timeout.
            RoochTransportError: If the polling request fails.
        """
        start_time = asyncio.get_event_loop().time()
        while True:
            try:
                tx_info = await self.get_transaction_by_hash(transaction_hash)

                if tx_info is not None:
                    # Check if the transaction execution is complete (or failed definitively)
                    # Based on openrpc, TransactionWithInfoView has execution_info
                    if tx_info.get("execution_info"):
                        # TODO: Check the actual status within execution_info if needed
                        # For now, assume presence of execution_info means it's processed.
                        return tx_info
                    # If no execution_info, it might still be pending or indexed differently.
                    # This simplistic check might need refinement based on node behavior.

            except RoochTransportError as e:
                # You might want to retry on certain errors or raise immediately on others
                print(f"Error polling transaction {transaction_hash}: {e}")
                # For simplicity, we raise here. Consider adding retry logic.
                raise

            # Check timeout
            elapsed = asyncio.get_event_loop().time() - start_time
            if elapsed >= timeout_secs:
                raise TimeoutError(
                    f"Timeout waiting for transaction {transaction_hash} after {timeout_secs} seconds"
                )

            # Wait before next poll
            await asyncio.sleep(poll_interval_ms / 1000.0)
    
    async def get_transactions(
        self, 
        cursor: int = 0, 
        limit: int = 25, 
        descending: bool = True
    ) -> Dict[str, Any]:
        """Get transactions with pagination
        
        Args:
            cursor: Starting cursor
            limit: Maximum number of transactions to return
            descending: Whether to return transactions in descending order
            
        Returns:
            List of transactions and pagination info
        """
        return await self._transport.request("rooch_getTransactions", [cursor, limit, descending])
    
    async def get_transactions_by_sender(
        self,
        sender: str,
        cursor: int = 0,
        limit: int = 25,
        descending: bool = True
    ) -> Dict[str, Any]:
        """Get transactions by sender with pagination
        
        Args:
            sender: Sender address
            cursor: Starting cursor
            limit: Maximum number of transactions to return
            descending: Whether to return transactions in descending order
            
        Returns:
            List of transactions and pagination info
        """
        return await self._transport.request(
            "rooch_getTransactionsBySender", 
            [sender, cursor, limit, descending]
        )
    
    async def dry_run_transaction(self, transaction: SignedTransaction) -> Dict[str, Any]:
        """Dry run a transaction without submitting it
        
        Args:
            transaction: Signed transaction
            
        Returns:
            Dry run results
        """
        # Serialize the transaction
        tx_bytes = TxSerializer.encode_transaction_for_submission(transaction)
        
        # Dry run the transaction
        return await self._transport.request("rooch_dryRunTransaction", [tx_bytes])
    
    async def execute_view_function(
        self,
        function_id: str,
        type_args: Optional[List[str]] = None,
        args: Optional[List[Any]] = None
    ) -> Any:
        """Execute a view function
        
        Args:
            function_id: Fully qualified function ID (e.g., 0x1::module::function)
            type_args: Optional list of type arguments as strings
            args: Optional list of function arguments (should be serializable to BCS)
            
        Returns:
            Function result
        """
        # Construct the FunctionCallView object as expected by the RPC spec
        function_call = {
            "function_id": function_id,
            "ty_args": type_args if type_args is not None else [],
            "args": args if args is not None else [] # Assuming args are already BCS encoded byte strings or similar
            # Note: Need to ensure 'args' are properly BCS encoded before calling this
            # The openrpc spec expects Vec<Vec<u8>> which implies each arg is a byte vector.
        }
        return await self._transport.request(
            "rooch_executeViewFunction",
            [function_call] # Pass the single FunctionCallView object
        )
    
    async def get_events_by_transaction_hash(self, transaction_hash: str) -> List[Dict[str, Any]]:
        """Get events by transaction hash
        
        Args:
            transaction_hash: Transaction hash
            
        Returns:
            List of events
        """
        return await self._transport.request("rooch_getEventsByTxHash", [transaction_hash])
    
    async def get_events_by_event_handle(
        self,
        event_handle_type: str,
        cursor: int = 0,
        limit: int = 25,
        descending: bool = True
    ) -> Dict[str, Any]:
        """Get events by event handle type
        
        Args:
            event_handle_type: Event handle type
            cursor: Starting cursor
            limit: Maximum number of events to return
            descending: Whether to return events in descending order
            
        Returns:
            List of events and pagination info
        """
        return await self._transport.request(
            "rooch_getEventsByEventHandle", 
            [event_handle_type, cursor, limit, descending]
        )