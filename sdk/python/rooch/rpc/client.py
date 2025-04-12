#!/usr/bin/env python3
# Copyright (c) Rooch Network
# SPDX-License-Identifier: Apache-2.0

"""JSON-RPC client for Rooch"""

import json
import httpx
from typing import Any, Dict, List, Optional, Union

from .types import RpcError

class JsonRpcClient:
    """JSON-RPC client for interacting with Rooch RPC server"""
    
    def __init__(self, endpoint: str = "http://localhost:50051/v1/jsonrpc"):
        """Initialize the JSON-RPC client
        
        Args:
            endpoint: URL of the RPC server
        """
        self.endpoint = endpoint
        self.client = httpx.Client()
        self.id_counter = 0
    
    def request(self, method: str, params: Any = None) -> Any:
        """Make a JSON-RPC request
        
        Args:
            method: RPC method name
            params: Method parameters
            
        Returns:
            Response result
            
        Raises:
            RpcError: If the server returns an error
        """
        # Create request payload
        self.id_counter += 1
        payload = {
            "jsonrpc": "2.0",
            "method": method,
            "params": params,
            "id": self.id_counter
        }
        
        try:
            # Send request
            response = self.client.post(self.endpoint, json=payload)
            
            # Parse response
            data = response.json()
            
            # Check for error
            if "error" in data:
                error = data["error"]
                code = error.get("code", 0)
                message = error.get("message", "Unknown error")
                error_data = error.get("data", None)
                raise RpcError(code, message, error_data)
                
            # Return result
            return data.get("result")
            
        except httpx.HTTPError as e:
            # Re-raise HTTP errors
            raise Exception(f"HTTP Connection Error: {str(e)}")
    
    def batch_request(self, requests: List[Dict[str, Any]], ignore_errors: bool = False) -> List[Any]:
        """Make a batch JSON-RPC request
        
        Args:
            requests: List of request objects with method and params
            ignore_errors: If True, return RpcError objects for errors instead of raising
            
        Returns:
            List of response results
            
        Raises:
            RpcError: If any request returns an error and ignore_errors is False
        """
        # Create request payloads
        payloads = []
        for request in requests:
            self.id_counter += 1
            payloads.append({
                "jsonrpc": "2.0",
                "method": request["method"],
                "params": request.get("params"),
                "id": self.id_counter
            })
        
        try:
            # Send batch request
            response = self.client.post(self.endpoint, json=payloads)
            
            # Parse responses
            data = response.json()
            
            # Process responses
            results = []
            for item in data:
                if "error" in item:
                    error = item["error"]
                    code = error.get("code", 0)
                    message = error.get("message", "Unknown error")
                    error_data = error.get("data", None)
                    error_obj = RpcError(code, message, error_data)
                    
                    if ignore_errors:
                        results.append(error_obj)
                    else:
                        raise error_obj
                else:
                    results.append(item.get("result"))
                    
            return results
            
        except httpx.HTTPError as e:
            # Re-raise HTTP errors
            raise Exception(f"HTTP Connection Error: {str(e)}")
    
    def __del__(self):
        """Clean up resources"""
        if hasattr(self, "client"):
            self.client.close()