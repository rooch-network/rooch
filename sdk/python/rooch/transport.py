#!/usr/bin/env python3
# Copyright (c) Rooch Network
# SPDX-License-Identifier: Apache-2.0

import json
import uuid
from typing import Any, Dict, List, Optional, Union

import aiohttp


class RoochTransportError(Exception):
    """Error during transport operations"""
    
    def __init__(self, message: str, code: Optional[int] = None, data: Any = None):
        """Initialize with error details
        
        Args:
            message: Error message
            code: Optional error code
            data: Optional error data
        """
        self.message = message
        self.code = code
        self.data = data
        
        super().__init__(self.message)
    
    @classmethod
    def from_response(cls, response: Dict[str, Any]) -> 'RoochTransportError':
        """Create from JSON-RPC error response
        
        Args:
            response: JSON-RPC error response
            
        Returns:
            Transport error
        """
        error = response.get("error", {})
        message = error.get("message", "Unknown error")
        code = error.get("code")
        data = error.get("data")
        print(f"RoochTransportError: {response}")
        return cls(message, code, data)


class RoochTransport:
    """Transport for communicating with a Rooch node"""
    
    def __init__(
        self, 
        url: str,
        request_timeout_ms: int = 30000,
        session: Optional[aiohttp.ClientSession] = None,
        headers: Optional[Dict[str, str]] = None
    ):
        """Initialize with endpoint URL
        
        Args:
            url: URL of the Rooch RPC endpoint
            request_timeout_ms: Request timeout in milliseconds
            session: Optional aiohttp client session
            headers: Optional additional HTTP headers
        """
        self.url = url
        self.request_timeout_ms = request_timeout_ms
        self.session = session
        self._should_close_session = session is None # Track if we created the session
        self.headers = headers or {}
        self.default_headers = {
            "Content-Type": "application/json",
            "User-Agent": "rooch-python-sdk"
        }
        
        # Add default headers
        for key, value in self.default_headers.items():
            if key not in self.headers:
                self.headers[key] = value
    
    async def request(self, method: str, params: Optional[List[Any]] = None) -> Any:
        """Make a JSON-RPC request
        
        Args:
            method: RPC method name
            params: RPC parameters
            
        Returns:
            RPC result
            
        Raises:
            RoochTransportError: If the request fails
        """
        # Prepare request payload
        payload = {
            "jsonrpc": "2.0",
            "id": str(uuid.uuid4()),
            "method": method,
            "params": params or []
        }
        
        # Create session if needed
        should_close_session = False
        session = self.session
        
        if session is None:
            should_close_session = True
            session = aiohttp.ClientSession()
            self.session = session # Store the created session
        
        try:
            # Make the request
            timeout = aiohttp.ClientTimeout(total=self.request_timeout_ms / 1000)
            async with session.post(
                self.url,
                json=payload,
                headers=self.headers,
                timeout=timeout
            ) as response:
                # Parse response
                try:
                    data = await response.json(content_type=None)
                except (json.JSONDecodeError, aiohttp.ContentTypeError):
                    text = await response.text()
                    raise RoochTransportError(f"Invalid JSON response: {text}")
                
                # Check for HTTP errors
                if response.status >= 400:
                    # Handle case where data might be None or not a dict
                    error_detail = "Unknown error"
                    if isinstance(data, dict):
                        error_detail = data.get('error', 'Unknown error')
                    elif data is not None:
                        error_detail = str(data) # Use string representation if not None or dict
                    raise RoochTransportError(
                        f"HTTP error {response.status}: {error_detail}"
                    )
                
                # Check for JSON-RPC errors
                if "error" in data:
                    raise RoochTransportError.from_response(data)
                
                # Return result
                if "result" not in data:
                    raise RoochTransportError("Missing 'result' in response")
                
                return data["result"]
        finally:
            # Close session if we created it *within this request*
            # The client-level close handles the case where the session was created 
            # during client initialization (if self._should_close_session is True)
            if should_close_session and session:
                 # Avoid closing the session if it was passed externally or managed by the client
                 if self.session == session: # Check if this is the main session managed by transport/client
                     pass # Client close will handle it if needed
                 else:
                    await session.close() # Close temporary session created just for this request


class RoochEnvironment:
    """Predefined Rooch network environments"""
    
    LOCAL = "http://localhost:50051"
    DEV = "https://dev-seed.rooch.network"
    TEST = "https://test-seed.rooch.network"
    MAIN = "https://main-seed.rooch.network"