#!/usr/bin/env python3
# Copyright (c) Rooch Network
# SPDX-License-Identifier: Apache-2.0

"""Tests for WebSocket transport and subscription functionality"""

import asyncio
import json
import pytest
import websockets
from unittest.mock import AsyncMock, MagicMock, patch

from rooch.client.ws_transport import RoochWebSocketTransport
from rooch.client.subscription_interface import Subscription


class MockWebSocket:
    """Mock WebSocket connection for testing"""
    
    def __init__(self, responses=None):
        """Initialize with optional mock responses
        
        Args:
            responses: Optional list of responses to return in order
        """
        self.responses = responses or []
        self.sent_messages = []
        self.closed = False
        self.response_index = 0
    
    async def send(self, message):
        """Mock send method
        
        Args:
            message: Message to send
        """
        self.sent_messages.append(json.loads(message))
    
    async def recv(self):
        """Mock receive method
        
        Returns:
            Next mock response
        """
        if self.response_index < len(self.responses):
            response = self.responses[self.response_index]
            self.response_index += 1
            return json.dumps(response)
        raise websockets.exceptions.ConnectionClosed(1000, "Mock connection closed")
    
    async def close(self, *args, **kwargs):
        """Mock close method"""
        self.closed = True


@pytest.mark.asyncio
async def test_websocket_transport_request():
    """Test WebSocket transport request method"""
    # Mock responses
    mock_responses = [
        {
            "jsonrpc": "2.0",
            "id": 1,
            "result": 42
        }
    ]
    
    # Create mock websocket
    mock_ws = MockWebSocket(mock_responses)
    
    # Patch websocket.connect to return our mock
    with patch("websockets.connect", AsyncMock(return_value=mock_ws)):
        # Create transport
        transport = RoochWebSocketTransport("ws://localhost:9944")
        
        # Make request
        result = await transport.request("test_method", ["param1", "param2"])
        
        # Verify result
        assert result == 42
        
        # Verify sent message format
        assert len(mock_ws.sent_messages) == 1
        assert mock_ws.sent_messages[0]["method"] == "test_method"
        assert mock_ws.sent_messages[0]["params"] == ["param1", "param2"]
        assert "id" in mock_ws.sent_messages[0]


@pytest.mark.asyncio
async def test_websocket_transport_error_handling():
    """Test WebSocket transport error handling"""
    # Mock error response
    mock_responses = [
        {
            "jsonrpc": "2.0",
            "id": 1,
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
        
        # Make request and expect exception
        with pytest.raises(Exception) as excinfo:
            await transport.request("invalid_method")
        
        # Verify exception message
        assert "Method not found" in str(excinfo.value)


@pytest.mark.asyncio
async def test_websocket_subscription():
    """Test WebSocket subscription functionality"""
    # Mock subscription responses
    mock_responses = [
        # Initial response with subscription ID
        {
            "jsonrpc": "2.0",
            "id": 1,
            "result": "subscription_123"
        },
        # First update message
        {
            "jsonrpc": "2.0",
            "method": "rooch_subscription",
            "params": {
                "subscription": "subscription_123",
                "result": {"event": "update1"}
            }
        },
        # Second update message
        {
            "jsonrpc": "2.0",
            "method": "rooch_subscription",
            "params": {
                "subscription": "subscription_123",
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
        
        # Start subscription
        subscription = await transport.subscribe(
            "test_subscription",
            ["param1"],
            on_message,
            on_error
        )
        
        # Verify subscription ID
        assert subscription.id == "subscription_123"
        
        # Wait for messages to be processed
        await asyncio.sleep(0.1)
        
        # Process pending messages manually for testing
        await transport._process_messages()
        
        # Verify received messages
        assert len(received_messages) == 2
        assert received_messages[0]["event"] == "update1"
        assert received_messages[1]["event"] == "update2"
        assert len(errors) == 0


@pytest.mark.asyncio
async def test_websocket_unsubscribe():
    """Test WebSocket unsubscribe functionality"""
    # Mock responses
    mock_responses = [
        # Initial response with subscription ID
        {
            "jsonrpc": "2.0",
            "id": 1,
            "result": "subscription_123"
        },
        # Unsubscribe response
        {
            "jsonrpc": "2.0",
            "id": 2,
            "result": True
        }
    ]
    
    # Create mock websocket
    mock_ws = MockWebSocket(mock_responses)
    
    # Patch websocket.connect to return our mock
    with patch("websockets.connect", AsyncMock(return_value=mock_ws)):
        # Create transport
        transport = RoochWebSocketTransport("ws://localhost:9944")
        
        # Start subscription
        subscription = await transport.subscribe(
            "test_subscription",
            ["param1"],
            lambda x: None,  # Dummy callback
            lambda x: None   # Dummy error handler
        )
        
        # Verify subscription ID
        assert subscription.id == "subscription_123"
        
        # Unsubscribe
        success = await transport.unsubscribe(subscription.id)
        
        # Verify result
        assert success is True
        
        # Verify sent message format for unsubscribe
        assert len(mock_ws.sent_messages) == 2
        assert mock_ws.sent_messages[1]["method"] == "rooch_unsubscribe"
        assert mock_ws.sent_messages[1]["params"] == ["subscription_123"]