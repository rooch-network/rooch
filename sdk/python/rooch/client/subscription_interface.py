#!/usr/bin/env python3
# Copyright (c) Rooch Network
# SPDX-License-Identifier: Apache-2.0

from abc import ABC, abstractmethod
from typing import Any, Callable, Dict, TypeVar

from .types.json_rpc import JsonRpcRequest

T = TypeVar('T')

class Subscription:
    """Represents a subscription to a real-time data stream"""
    
    def __init__(self, id: str, unsubscribe_fn: Callable[[], None]):
        """
        Args:
            id: Subscription ID
            unsubscribe_fn: Function to call to unsubscribe
        """
        self.id = id
        self._unsubscribe_fn = unsubscribe_fn
    
    def unsubscribe(self) -> None:
        """Unsubscribe from the real-time data stream"""
        if self._unsubscribe_fn:
            self._unsubscribe_fn()

class RoochSubscriptionTransport(ABC):
    """Interface for subscription-based transport to communicate with Rooch nodes"""
    
    @abstractmethod
    async def subscribe(self, request: JsonRpcRequest) -> Subscription:
        """Subscribe to a specific request
        
        Args:
            request: JSON-RPC request object containing method and parameters
            
        Returns:
            A Subscription object containing the subscription ID and an unsubscribe method
        """
        pass
    
    @abstractmethod
    def unsubscribe(self, subscription_id: str) -> None:
        """Unsubscribe from a specific subscription
        
        Args:
            subscription_id: The subscription ID
        """
        pass
    
    @abstractmethod
    def on_message(self, callback: Callable[[Any], None]) -> None:
        """Register a callback to handle subscription events
        
        Args:
            callback: Function to handle subscription events
        """
        pass
    
    @abstractmethod
    def on_reconnected(self, callback: Callable[[], None]) -> None:
        """Register a callback to handle reconnection events
        
        Args:
            callback: Function called when the connection is re-established
        """
        pass
    
    @abstractmethod
    def on_error(self, callback: Callable[[Exception], None]) -> None:
        """Register a callback to handle transport-level errors
        
        Args:
            callback: Function to handle errors
        """
        pass
    
    @abstractmethod
    def destroy(self) -> None:
        """Clean up resources and close the connection"""
        pass