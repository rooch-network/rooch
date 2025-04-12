#!/usr/bin/env python3
# Copyright (c) Rooch Network
# SPDX-License-Identifier: Apache-2.0

import json
import uuid
from typing import Any, Dict, List, Optional

import aiohttp
import requests

from ..utils.logger import get_logger
from .transport_interface import RoochTransport
from .error import RoochTransportError

logger = get_logger("http-transport")

class RoochHTTPTransport(RoochTransport):
    """HTTP transport implementation for Rooch client"""
    
    def __init__(self, url: str, timeout_ms: int = 30000):
        """Initialize the HTTP transport
        
        Args:
            url: URL of the Rooch node
            timeout_ms: Request timeout in milliseconds
        """
        self.url = url
        self.timeout_sec = timeout_ms / 1000  # Convert to seconds for the libraries
        self._session = None
    
    async def _get_session(self) -> aiohttp.ClientSession:
        """Get or create an aiohttp session
        
        Returns:
            An aiohttp client session
        """
        if self._session is None or self._session.closed:
            self._session = aiohttp.ClientSession(
                timeout=aiohttp.ClientTimeout(total=self.timeout_sec)
            )
        return self._session
    
    async def request(self, method: str, params: List[Any]) -> Any:
        """Send a request to the Rooch node
        
        Args:
            method: The JSON-RPC method name
            params: The parameters to pass to the method
            
        Returns:
            The response from the node
            
        Raises:
            RoochTransportError: If the request fails
        """
        request_id = str(uuid.uuid4())
        payload = {
            "jsonrpc": "2.0",
            "id": request_id,
            "method": method,
            "params": params
        }
        
        logger.debug(f"Sending request: {json.dumps(payload)}")
        
        try:
            session = await self._get_session()
            async with session.post(self.url, json=payload) as response:
                if response.status != 200:
                    error_text = await response.text()
                    raise RoochTransportError(
                        f"HTTP Error {response.status}: {error_text}"
                    )
                
                result = await response.json()
                
                if "error" in result:
                    error = result["error"]
                    raise RoochTransportError(
                        f"RPC Error {error.get('code')}: {error.get('message')}",
                        code=error.get("code", 0),
                        data=error.get("data")
                    )
                    
                return result["result"]
        except aiohttp.ClientError as e:
            raise RoochTransportError(f"HTTP Client Error: {str(e)}") from e
        except json.JSONDecodeError as e:
            raise RoochTransportError(f"Invalid JSON in response: {str(e)}") from e
        except Exception as e:
            raise RoochTransportError(f"Unexpected error in HTTP transport: {str(e)}") from e
    
    def destroy(self) -> None:
        """Clean up resources and close connections"""
        if self._session and not self._session.closed:
            if self._session.loop.is_running():
                self._session.loop.create_task(self._session.close())
            else:
                import asyncio
                asyncio.run(self._session.close())
            self._session = None