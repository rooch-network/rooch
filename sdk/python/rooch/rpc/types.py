#!/usr/bin/env python3
# Copyright (c) Rooch Network
# SPDX-License-Identifier: Apache-2.0

"""RPC error types for Rooch Python SDK"""

class RpcError(Exception):
    """JSON-RPC error response exception"""
    
    def __init__(self, code: int, message: str, data=None):
        """Initialize RPC error
        
        Args:
            code: Error code
            message: Error message
            data: Additional error data
        """
        self.code = code
        self.message = message
        self.data = data
        super().__init__(f"RPC Error {code}: {message}" + (f" - {data}" if data else ""))
    
    def __repr__(self) -> str:
        """Return string representation of RPC error"""
        return f"RpcError({self.code}, '{self.message}', {self.data})"