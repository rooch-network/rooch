#!/usr/bin/env python3
# Copyright (c) Rooch Network
# SPDX-License-Identifier: Apache-2.0

from abc import ABC, abstractmethod
from typing import Any, Dict, List, TypeVar, Optional

T = TypeVar('T')

class RoochTransportRequestOptions:
    """Options for a JSON-RPC request"""
    
    def __init__(self, method: str, params: List[Any]):
        self.method = method
        self.params = params

class RoochTransport(ABC):
    """Interface for transport layer to communicate with Rooch nodes"""
    
    @abstractmethod
    async def request(self, method: str, params: List[Any]) -> Any:
        """Send a request to the Rooch node
        
        Args:
            method: The JSON-RPC method name
            params: The parameters to pass to the method
            
        Returns:
            The response from the node
        """
        pass
    
    @abstractmethod
    def destroy(self) -> None:
        """Clean up resources and close connections
        
        Should be called when the transport is no longer needed
        """
        pass