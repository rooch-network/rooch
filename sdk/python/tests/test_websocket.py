#!/usr/bin/env python3
# Copyright (c) Rooch Network
# SPDX-License-Identifier: Apache-2.0

"""Tests for WebSocket transport and subscription functionality"""

import asyncio
import json
import pytest
import websockets
from unittest.mock import AsyncMock, MagicMock, patch
import uuid
from typing import List, Dict, Any

from rooch.client.ws_transport import RoochWebSocketTransport
from rooch.client.subscription_interface import Subscription
from rooch.client.error import RoochTransportError, RoochSubscriptionError
from rooch.client.types.json_rpc import JsonRpcRequest


class MockWebSocket:
    """Mock WebSocket connection for testing"""
    
    def __init__(self, responses: List[Dict[str, Any]]):
        """Initialize with optional mock responses
        
        Args:
            responses: Optional list of responses to return in order
        """
        self._responses = [json.dumps(r) for r in responses]
        self._sent_messages = []
        self.closed = False
        self._iter = iter(self._responses)
    
    async def send(self, message: str) -> None:
        """Mock send method
        
        Args:
            message: Message to send
        """
        self._sent_messages.append(json.loads(message))
    
    async def recv(self) -> str:
        """Mock receive method
        
        Returns:
            Next mock response
        """
        try:
            return next(self._iter)
        except StopIteration:
            # Keep the connection open but don't return more messages
            await asyncio.sleep(3600) # Sleep indefinitely
            raise StopIteration # Should not be reached
    
    async def close(self) -> None:
        """Mock close method"""
        self.closed = True

    # Add __aiter__ to be compatible with async for
    def __aiter__(self):
        return self

    # Add __anext__ to be compatible with async for
    async def __anext__(self):
        try:
            # Simulate receiving messages one by one
            return next(self._iter)
        except StopIteration:
            # Stop iteration when no more mock responses
            raise StopAsyncIteration


@pytest.mark.asyncio
async def test_websocket_transport_request():
    """Test WebSocket transport request method"""
    # Mock responses
    mock_responses = [
        {
            "jsonrpc": "2.0",
            "id": 1, # Assuming the generated uuid matches this, might need adjustment
            "result": 42
        }
    ]
    
    # Create mock websocket
    mock_ws = MockWebSocket(mock_responses)
    
    # Patch websocket.connect to return our mock
    with patch("websockets.connect", AsyncMock(return_value=mock_ws)):
        # Create transport
        transport = RoochWebSocketTransport("ws://localhost:9944")
        
        # We need to mock uuid.uuid4() if we want to match the ID in mock_responses
        test_uuid = "test-uuid-1"
        with patch("uuid.uuid4", return_value=test_uuid):
            # Make request
            result = await transport.request("test_method", ["param1", "param2"])
            
            # Verify result
            assert result == 42
            
            # Verify message sent
            assert len(mock_ws._sent_messages) == 1
            sent_msg = mock_ws._sent_messages[0]
            assert sent_msg["method"] == "test_method"
            assert sent_msg["params"] == ["param1", "param2"]
            assert sent_msg["id"] == test_uuid
        
        await transport.destroy() # Clean up transport


@pytest.mark.asyncio
async def test_websocket_transport_error_handling():
    """Test WebSocket transport error handling"""
    # Mock error response
    test_uuid = "test-uuid-error"
    mock_responses = [
        {
            "jsonrpc": "2.0",
            "id": test_uuid,
            "error": {
                "code": -32601,
                "message": "Method not found"
            }
        }
    ]
    
    # Create mock websocket
    mock_ws = MockWebSocket(mock_responses)
    
    # Patch websocket.connect to return our mock
    with patch("websockets.connect", AsyncMock(return_value=mock_ws)):
        # Create transport
        transport = RoochWebSocketTransport("ws://localhost:9944")
        
        with patch("uuid.uuid4", return_value=test_uuid):
            # Make request and expect exception
            with pytest.raises(RoochTransportError) as excinfo:
                # Provide empty list for params as it's required by the method signature
                await transport.request("invalid_method", [])
            
            # Verify exception message
            assert "Method not found" in str(excinfo.value)
            assert excinfo.value.code == -32601
        
        await transport.destroy()


@pytest.mark.asyncio
async def test_websocket_subscription():
    """Test WebSocket subscription functionality"""
    sub_req_uuid = "sub-req-uuid"
    sub_id = "subscription_123"
    
    # Mock subscription responses
    mock_responses = [
        # Initial response with subscription ID
        {
            "jsonrpc": "2.0",
            "id": sub_req_uuid, 
            "result": sub_id
        },
        # First update message
        {
            "jsonrpc": "2.0",
            "method": "rooch_subscription", # Assuming this is the notification method
            "params": {
                "subscription": sub_id,
                "result": {"event": "update1"}
            }
        },
        # Second update message
        {
            "jsonrpc": "2.0",
            "method": "rooch_subscription",
            "params": {
                "subscription": sub_id,
                "result": {"event": "update2"}
            }
        }
    ]
    
    # Create mock websocket
    mock_ws = MockWebSocket(mock_responses)
    
    # Callbacks for testing
    received_messages = []
    errors = []
    
    def on_message(message):
        received_messages.append(message)
    
    def on_error(error):
        errors.append(error)
    
    # Patch websocket.connect to return our mock
    with patch("websockets.connect", AsyncMock(return_value=mock_ws)):
        # Create transport
        transport = RoochWebSocketTransport("ws://localhost:9944")
        
        # Register callbacks
        transport.on_message(on_message)
        transport.on_error(on_error)

        # Create subscription request object
        subscribe_request = JsonRpcRequest(method="test_subscription", params=["param1"])

        with patch("uuid.uuid4", return_value=sub_req_uuid):
            # Start subscription
            subscription = await transport.subscribe(subscribe_request)
            assert subscription.id == sub_id
            assert callable(subscription.unsubscribe)

        # Allow time for messages to be processed by the background task
        await asyncio.sleep(0.1)

        # Verify messages received via callback
        assert len(received_messages) == 2
        assert received_messages[0]["params"]["result"] == {"event": "update1"}
        assert received_messages[1]["params"]["result"] == {"event": "update2"}
        assert len(errors) == 0
        
        await transport.destroy()


@pytest.mark.asyncio
async def test_websocket_unsubscribe():
    """Test WebSocket unsubscribe functionality"""
    sub_req_uuid = "sub-req-uuid"
    unsub_req_uuid = "unsub-req-uuid"
    sub_id = "subscription_123"
    
    # Mock responses
    mock_responses = [
        # Initial response with subscription ID
        {
            "jsonrpc": "2.0",
            "id": sub_req_uuid,
            "result": sub_id
        },
        # Unsubscribe response
        {
            "jsonrpc": "2.0",
            "id": unsub_req_uuid,
            "result": True
        }
    ]
    
    # Create mock websocket
    mock_ws = MockWebSocket(mock_responses)
    
    # Patch websocket.connect to return our mock
    with patch("websockets.connect", AsyncMock(return_value=mock_ws)):
        # Create transport
        transport = RoochWebSocketTransport("ws://localhost:9944")
        
        # Register dummy callbacks (needed for subscribe)
        transport.on_message(lambda x: None)
        transport.on_error(lambda x: None)

        # Create subscription request object
        subscribe_request = JsonRpcRequest(method="test_subscription", params=["param1"])

        # Start subscription
        with patch("uuid.uuid4", return_value=sub_req_uuid):
            subscription = await transport.subscribe(subscribe_request)
            assert subscription.id == sub_id

        # Unsubscribe
        with patch("uuid.uuid4", return_value=unsub_req_uuid):
            await subscription.unsubscribe() 

        # Allow time for unsubscribe message to be sent
        await asyncio.sleep(0.1)

        # Verify unsubscribe message was sent
        assert len(mock_ws._sent_messages) == 2 # Subscribe + Unsubscribe
        unsubscribe_msg = mock_ws._sent_messages[1]
        assert unsubscribe_msg["method"] == "unsubscribe_method" # Replace with actual unsubscribe method name
        assert unsubscribe_msg["params"] == [sub_id]
        assert unsubscribe_msg["id"] == unsub_req_uuid
        
        await transport.destroy()