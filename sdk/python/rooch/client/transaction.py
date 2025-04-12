#!/usr/bin/env python3
# Copyright (c) Rooch Network
# SPDX-License-Identifier: Apache-2.0

from typing import Any, Dict, List, Optional, Union

from ..transport import RoochTransport
from ..transactions.serializer import TxSerializer
from ..transactions.types import SignedTransaction


class TransactionClient:
    """Client for Rooch transaction operations"""
    
    def __init__(self, transport: RoochTransport):
        """Initialize with a transport
        
        Args:
            transport: Transport for communicating with the Rooch node
        """
        self._transport = transport
    
    async def submit_transaction(self, transaction: SignedTransaction) -> str:
        """Submit a signed transaction
        
        Args:
            transaction: Signed transaction
            
        Returns:
            Transaction hash
        """
        # Serialize the transaction
        tx_bytes = TxSerializer.encode_transaction_for_submission(transaction)
        
        # Submit the transaction
        return await self._transport.request("rooch_submitTransaction", [tx_bytes])
    
    async def get_transaction_by_hash(self, transaction_hash: str) -> Dict[str, Any]:
        """Get transaction by hash
        
        Args:
            transaction_hash: Transaction hash
            
        Returns:
            Transaction information
        """
        return await self._transport.request("rooch_getTransactionByHash", [transaction_hash])
    
    async def get_transaction_info_by_hash(self, transaction_hash: str) -> Dict[str, Any]:
        """Get transaction info by hash
        
        Args:
            transaction_hash: Transaction hash
            
        Returns:
            Transaction info
        """
        return await self._transport.request("rooch_getTransactionInfoByHash", [transaction_hash])
    
    async def wait_for_transaction(
        self, 
        transaction_hash: str, 
        timeout_secs: int = 60, 
        poll_interval_ms: int = 1000
    ) -> Dict[str, Any]:
        """Wait for transaction to be confirmed
        
        Args:
            transaction_hash: Transaction hash
            timeout_secs: Timeout in seconds
            poll_interval_ms: Polling interval in milliseconds
            
        Returns:
            Transaction information when confirmed
        """
        return await self._transport.request(
            "rooch_waitForTransaction", 
            [transaction_hash, timeout_secs, poll_interval_ms]
        )
    
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
        type_args: List[str],
        args: List[Any]
    ) -> Any:
        """Execute a view function
        
        Args:
            function_id: Function ID (module::function)
            type_args: Type arguments
            args: Function arguments
            
        Returns:
            Function result
        """
        return await self._transport.request(
            "rooch_executeViewFunction", 
            [function_id, type_args, args]
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