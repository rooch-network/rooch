#!/usr/bin/env python3
# Copyright (c) Rooch Network
# SPDX-License-Identifier: Apache-2.0

from typing import Any, Dict, List, Optional, Union


class JsonRpcRequest:
    """JSON-RPC 2.0 request object"""
    
    def __init__(self, method: str, params: List[Any], id: Optional[str] = None):
        """
        Args:
            method: JSON-RPC method name
            params: Parameters to pass to the method
            id: Optional request ID (generated if not provided)
        """
        self.method = method
        self.params = params
        self.id = id
    
    def to_dict(self) -> Dict[str, Any]:
        """Convert request to a dictionary format suitable for JSON serialization
        
        Returns:
            Dictionary representation of the request
        """
        return {
            "jsonrpc": "2.0",
            "id": self.id,
            "method": self.method,
            "params": self.params
        }


class JsonRpcError:
    """JSON-RPC 2.0 error object"""
    
    def __init__(self, code: int, message: str, data: Optional[Any] = None):
        """
        Args:
            code: Error code
            message: Error message
            data: Optional additional error details
        """
        self.code = code
        self.message = message
        self.data = data
    
    @classmethod
    def from_dict(cls, error_dict: Dict[str, Any]) -> 'JsonRpcError':
        """Create an error object from a dictionary
        
        Args:
            error_dict: Dictionary representation of an error
            
        Returns:
            JsonRpcError instance
        """
        return cls(
            code=error_dict.get("code", 0),
            message=error_dict.get("message", "Unknown error"),
            data=error_dict.get("data")
        )
    
    def to_dict(self) -> Dict[str, Any]:
        """Convert error to a dictionary format suitable for JSON serialization
        
        Returns:
            Dictionary representation of the error
        """
        result = {
            "code": self.code,
            "message": self.message,
        }
        if self.data is not None:
            result["data"] = self.data
        return result


class JsonRpcResponse:
    """JSON-RPC 2.0 response object"""
    
    def __init__(
        self, 
        id: Optional[str], 
        result: Optional[Any] = None, 
        error: Optional[JsonRpcError] = None
    ):
        """
        Args:
            id: Request ID
            result: Response result (if successful)
            error: Error object (if request failed)
        """
        self.id = id
        self.result = result
        self.error = error
    
    @classmethod
    def from_dict(cls, response_dict: Dict[str, Any]) -> 'JsonRpcResponse':
        """Create a response object from a dictionary
        
        Args:
            response_dict: Dictionary representation of a response
            
        Returns:
            JsonRpcResponse instance
        """
        id = response_dict.get("id")
        result = response_dict.get("result")
        error = response_dict.get("error")
        
        error_obj = None
        if error:
            error_obj = JsonRpcError.from_dict(error)
            
        return cls(id=id, result=result, error=error_obj)
    
    def to_dict(self) -> Dict[str, Any]:
        """Convert response to a dictionary format suitable for JSON serialization
        
        Returns:
            Dictionary representation of the response
        """
        result = {
            "jsonrpc": "2.0",
            "id": self.id,
        }
        
        if self.error:
            result["error"] = self.error.to_dict()
        else:
            result["result"] = self.result
            
        return result