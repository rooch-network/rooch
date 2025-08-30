#!/usr/bin/env python3
# Copyright (c) Rooch Network
# SPDX-License-Identifier: Apache-2.0

import asyncio
import json
import uuid
from typing import Any, Callable, Dict, List, Optional, Set

import websockets
from websockets.exceptions import WebSocketException

from ..utils.logger import get_logger
from .error import RoochSubscriptionError, RoochTransportError
from .subscription_interface import RoochSubscriptionTransport, Subscription
from .transport_interface import RoochTransport
from .types.json_rpc import JsonRpcRequest

logger = get_logger("ws-transport")


class RoochWebSocketTransport(RoochTransport, RoochSubscriptionTransport):
    """WebSocket transport implementation for Rooch client"""
    
    def __init__(self, url: str, auto_connect: bool = True, reconnect_delay: int = 5000):
        """Initialize the WebSocket transport
        
        Args:
            url: WebSocket URL of the Rooch node
            auto_connect: Whether to connect automatically on initialization
            reconnect_delay: Delay in milliseconds between reconnection attempts
        """
        self.url = url
        self.auto_connect = auto_connect
        self.reconnect_delay = reconnect_delay / 1000  # Convert to seconds
        
        self._connection = None
        self._connected = False
        self._message_callbacks: List[Callable[[Any], None]] = []
        self._reconnect_callbacks: List[Callable[[], None]] = []
        self._error_callbacks: List[Callable[[Exception], None]] = []
        self._pending_requests: Dict[str, asyncio.Future] = {}
        self._active_subscriptions: Dict[str, Dict[str, Any]] = {}
        self._receive_task = None
        self._reconnect_task = None
        self._lock = asyncio.Lock()
        self._closed = False
        
        if auto_connect:
            # Create a background task to connect
            asyncio.create_task(self.connect())
    
    async def connect(self) -> None:
        """Connect to the WebSocket server
        
        Raises:
            RoochTransportError: If the connection fails
        """
        if self._connected:
            return
        
        try:
            async with self._lock:
                if self._connected:
                    return
                    
                logger.info(f"Connecting to WebSocket at {self.url}")
                self._connection = await websockets.connect(
                    self.url,
                    ping_interval=20,
                    ping_timeout=20,
                    close_timeout=5,
                )
                self._connected = True
                
                # Start listening for messages
                if self._receive_task is None or self._receive_task.done():
                    self._receive_task = asyncio.create_task(self._receive_messages())
                
                logger.info("WebSocket connection established")
        except WebSocketException as e:
            logger.error(f"Failed to connect to WebSocket: {e}")
            self._handle_error(RoochTransportError(f"WebSocket connection failed: {str(e)}"))
            
            if not self._closed and self._reconnect_task is None:
                self._reconnect_task = asyncio.create_task(self._reconnect())
    
    async def _reconnect(self) -> None:
        """Reconnect to the WebSocket server with exponential backoff"""
        retry_delay = self.reconnect_delay
        max_delay = 60  # Maximum delay in seconds
        
        while not self._closed:
            logger.info(f"Reconnecting in {retry_delay} seconds...")
            await asyncio.sleep(retry_delay)
            
            try:
                await self.connect()
                
                if self._connected:
                    logger.info("Reconnection successful")
                    # Notify callbacks that we've reconnected
                    for callback in self._reconnect_callbacks:
                        try:
                            callback()
                        except Exception as e:
                            logger.error(f"Error in reconnect callback: {e}")
                    break
            except Exception as e:
                logger.error(f"Reconnection attempt failed: {e}")
                # Increase delay with exponential backoff, capped at max_delay
                retry_delay = min(retry_delay * 1.5, max_delay)
        
        self._reconnect_task = None
    
    async def _receive_messages(self) -> None:
        """Background task to receive and process WebSocket messages"""
        if not self._connection:
            return
        
        try:
            async for message in self._connection:
                try:
                    data = json.loads(message)
                    logger.debug(f"Received WebSocket message: {data}")
                    
                    # Handle responses to pending requests
                    if "id" in data and data["id"] in self._pending_requests:
                        future = self._pending_requests.pop(data["id"])
                        if "error" in data:
                            error = data["error"]
                            future.set_exception(
                                RoochTransportError(
                                    f"RPC Error {error.get('code')}: {error.get('message')}",
                                    code=error.get("code", 0),
                                    data=error.get("data")
                                )
                            )
                        else:
                            future.set_result(data.get("result"))
                    
                    # Handle subscription notifications
                    elif "method" in data and data["method"].startswith("rooch_subscribe"):
                        # Process subscription message
                        for callback in self._message_callbacks:
                            try:
                                callback(data)
                            except Exception as e:
                                logger.error(f"Error in message callback: {e}")
                    
                    # Other messages
                    else:
                        logger.debug(f"Received unhandled message: {data}")
                except json.JSONDecodeError as e:
                    logger.error(f"Invalid JSON in WebSocket message: {e}")
        except websockets.exceptions.ConnectionClosed as e:
            logger.warning(f"WebSocket connection closed: {e}")
            self._connected = False
            
            # Reject all pending requests
            for req_id, future in list(self._pending_requests.items()):
                self._pending_requests.pop(req_id)
                future.set_exception(
                    RoochTransportError("WebSocket connection closed")
                )
            
            # Try to reconnect if not explicitly closed
            if not self._closed and self._reconnect_task is None:
                self._reconnect_task = asyncio.create_task(self._reconnect())
        except Exception as e:
            logger.error(f"Error in WebSocket message loop: {e}")
            self._connected = False
            self._handle_error(e)
    
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
        if not self._connected:
            await self.connect()
            
        if not self._connected:
            raise RoochTransportError("Not connected to WebSocket server")
            
        request_id = str(uuid.uuid4())
        payload = {
            "jsonrpc": "2.0",
            "id": request_id,
            "method": method,
            "params": params
        }
        
        logger.debug(f"Sending WebSocket request: {payload}")
        
        future = asyncio.Future()
        self._pending_requests[request_id] = future
        
        try:
            json_payload = json.dumps(payload)
            await self._connection.send(json_payload)
            return await asyncio.wait_for(future, timeout=30)
        except asyncio.TimeoutError:
            self._pending_requests.pop(request_id, None)
            raise RoochTransportError(f"Request timed out: {method}")
        except Exception as e:
            self._pending_requests.pop(request_id, None)
            raise RoochTransportError(f"WebSocket request failed: {str(e)}") from e
    
    async def subscribe(self, request: JsonRpcRequest) -> Subscription:
        """Subscribe to a specific request
        
        Args:
            request: JSON-RPC request object containing method and parameters
            
        Returns:
            A Subscription object containing the subscription ID and an unsubscribe method
            
        Raises:
            RoochSubscriptionError: If the subscription fails
        """
        if not self._connected:
            await self.connect()
            
        if not self._connected:
            raise RoochSubscriptionError("Not connected to WebSocket server")
        
        try:
            # Send the subscription request
            subscription_id = await self.request(request.method, request.params)
            
            if not isinstance(subscription_id, str):
                raise RoochSubscriptionError(
                    f"Invalid subscription ID: {subscription_id}"
                )
            
            # Store subscription details for potential resubscription
            self._active_subscriptions[subscription_id] = {
                "method": request.method,
                "params": request.params
            }
            
            # Create subscription object with unsubscribe function
            def unsubscribe_fn():
                self.unsubscribe(subscription_id)
            
            return Subscription(subscription_id, unsubscribe_fn)
        except Exception as e:
            raise RoochSubscriptionError(f"Failed to create subscription: {str(e)}") from e
    
    def unsubscribe(self, subscription_id: str) -> None:
        """Unsubscribe from a specific subscription
        
        Args:
            subscription_id: The subscription ID
        """
        if not self._connected:
            logger.warning(f"Cannot unsubscribe {subscription_id}: not connected")
            return
        
        # Remove from active subscriptions
        self._active_subscriptions.pop(subscription_id, None)
        
        # Create unsubscribe task
        method = "rooch_unsubscribe"
        params = [subscription_id]
        
        async def do_unsubscribe():
            try:
                await self.request(method, params)
                logger.debug(f"Unsubscribed from {subscription_id}")
            except Exception as e:
                logger.error(f"Failed to unsubscribe from {subscription_id}: {e}")
        
        asyncio.create_task(do_unsubscribe())
    
    def on_message(self, callback: Callable[[Any], None]) -> None:
        """Register a callback to handle subscription events
        
        Args:
            callback: Function to handle subscription events
        """
        if callback not in self._message_callbacks:
            self._message_callbacks.append(callback)
    
    def on_reconnected(self, callback: Callable[[], None]) -> None:
        """Register a callback to handle reconnection events
        
        Args:
            callback: Function called when the connection is re-established
        """
        if callback not in self._reconnect_callbacks:
            self._reconnect_callbacks.append(callback)
    
    def on_error(self, callback: Callable[[Exception], None]) -> None:
        """Register a callback to handle transport-level errors
        
        Args:
            callback: Function to handle errors
        """
        if callback not in self._error_callbacks:
            self._error_callbacks.append(callback)
    
    def _handle_error(self, error: Exception) -> None:
        """Process an error and notify all registered error callbacks
        
        Args:
            error: The error that occurred
        """
        for callback in self._error_callbacks:
            try:
                callback(error)
            except Exception as e:
                logger.error(f"Error in error callback: {e}")
    
    async def _resubscribe_all(self) -> None:
        """Resubscribe to all active subscriptions after a reconnection"""
        if not self._connected:
            logger.warning("Cannot resubscribe: not connected")
            return
        
        logger.info(f"Resubscribing to {len(self._active_subscriptions)} active subscriptions")
        
        old_subscriptions = self._active_subscriptions.copy()
        self._active_subscriptions.clear()
        
        for old_id, sub_info in old_subscriptions.items():
            try:
                request = JsonRpcRequest(
                    method=sub_info["method"],
                    params=sub_info["params"]
                )
                await self.subscribe(request)
                logger.debug(f"Resubscribed to {old_id}")
            except Exception as e:
                logger.error(f"Failed to resubscribe to {old_id}: {e}")
    
    def destroy(self) -> None:
        """Clean up resources and close the connection"""
        self._closed = True
        
        # Cancel background tasks
        if self._receive_task and not self._receive_task.done():
            self._receive_task.cancel()
        
        if self._reconnect_task and not self._reconnect_task.done():
            self._reconnect_task.cancel()
        
        # Clear callbacks
        self._message_callbacks.clear()
        self._reconnect_callbacks.clear()
        self._error_callbacks.clear()
        
        # Close WebSocket connection
        if self._connection:
            asyncio.create_task(self._connection.close())
            self._connection = None
            
        self._connected = False