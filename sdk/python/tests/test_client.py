#!/usr/bin/env python3
# Copyright (c) Rooch Network
# SPDX-License-Identifier: Apache-2.0

"""Tests for Client module"""

import pytest
from unittest.mock import patch, MagicMock
import json
from rooch.rpc.client import JsonRpcClient
from rooch.rpc.types import RpcError

class TestJsonRpcClient:
    """Tests for JsonRpcClient class"""
    
    def test_init_client(self):
        """Test initializing client with different URLs"""
        # Test with default URL
        client = JsonRpcClient()
        assert client.endpoint == "http://localhost:50051/v1/jsonrpc"
        
        # Test with custom URL
        custom_url = "https://api.rooch.network/v1/jsonrpc"
        client = JsonRpcClient(endpoint=custom_url)
        assert client.endpoint == custom_url
    
    @patch('rooch.rpc.client.httpx.Client.post')
    def test_request_success(self, mock_post):
        """Test successful JSON-RPC request"""
        # Setup mock response
        mock_response = MagicMock()
        mock_response.status_code = 200
        mock_response.json.return_value = {
            "jsonrpc": "2.0",
            "id": 1,
            "result": {"value": "test_result"}
        }
        mock_post.return_value = mock_response
        
        # Create client and make request
        client = JsonRpcClient()
        result = client.request("test_method", {"param1": "value1"})
        
        # Verify request was made correctly
        mock_post.assert_called_once()
        args, kwargs = mock_post.call_args
        assert kwargs["json"]["method"] == "test_method"
        assert kwargs["json"]["params"] == {"param1": "value1"}
        assert kwargs["json"]["jsonrpc"] == "2.0"
        assert "id" in kwargs["json"]
        
        # Verify result
        assert result == {"value": "test_result"}
    
    @patch('rooch.rpc.client.httpx.Client.post')
    def test_request_error(self, mock_post):
        """Test JSON-RPC request with error response"""
        # Setup mock response with error
        mock_response = MagicMock()
        mock_response.status_code = 200
        mock_response.json.return_value = {
            "jsonrpc": "2.0",
            "id": 1,
            "error": {
                "code": -32600,
                "message": "Invalid Request",
                "data": "Additional error data"
            }
        }
        mock_post.return_value = mock_response
        
        # Create client and verify error is raised
        client = JsonRpcClient()
        with pytest.raises(RpcError) as exc_info:
            client.request("test_method", {"param1": "value1"})
        
        # Verify error details
        assert exc_info.value.code == -32600
        assert exc_info.value.message == "Invalid Request"
        assert exc_info.value.data == "Additional error data"
    
    @patch('rooch.rpc.client.httpx.Client.post')
    def test_http_error(self, mock_post):
        """Test handling HTTP errors"""
        # Setup mock response with HTTP error
        mock_post.side_effect = Exception("HTTP Connection Error")
        
        # Create client and verify error is raised
        client = JsonRpcClient()
        with pytest.raises(Exception) as exc_info:
            client.request("test_method", {"param1": "value1"})
        
        assert str(exc_info.value) == "HTTP Connection Error"
    
    @patch('rooch.rpc.client.httpx.Client.post')
    def test_batch_request(self, mock_post):
        """Test batch JSON-RPC request"""
        # Setup mock response
        mock_response = MagicMock()
        mock_response.status_code = 200
        mock_response.json.return_value = [
            {
                "jsonrpc": "2.0",
                "id": 1,
                "result": {"value": "result1"}
            },
            {
                "jsonrpc": "2.0",
                "id": 2,
                "result": {"value": "result2"}
            }
        ]
        mock_post.return_value = mock_response
        
        # Create client and make batch request
        client = JsonRpcClient()
        requests = [
            {"method": "method1", "params": {"param1": "value1"}},
            {"method": "method2", "params": {"param2": "value2"}}
        ]
        results = client.batch_request(requests)
        
        # Verify request was made correctly
        mock_post.assert_called_once()
        args, kwargs = mock_post.call_args
        assert isinstance(kwargs["json"], list)
        assert len(kwargs["json"]) == 2
        assert kwargs["json"][0]["method"] == "method1"
        assert kwargs["json"][1]["method"] == "method2"
        
        # Verify results
        assert len(results) == 2
        assert results[0] == {"value": "result1"}
        assert results[1] == {"value": "result2"}
    
    @patch('rooch.rpc.client.httpx.Client.post')
    def test_batch_request_with_errors(self, mock_post):
        """Test batch JSON-RPC request with mixed success and errors"""
        # Setup mock response
        mock_response = MagicMock()
        mock_response.status_code = 200
        mock_response.json.return_value = [
            {
                "jsonrpc": "2.0",
                "id": 1,
                "result": {"value": "result1"}
            },
            {
                "jsonrpc": "2.0",
                "id": 2,
                "error": {
                    "code": -32600,
                    "message": "Invalid Request",
                    "data": None
                }
            }
        ]
        mock_post.return_value = mock_response
        
        # Create client and make batch request
        client = JsonRpcClient()
        requests = [
            {"method": "method1", "params": {"param1": "value1"}},
            {"method": "method2", "params": {"param2": "value2"}}
        ]
        
        # Verify error is raised
        with pytest.raises(RpcError) as exc_info:
            client.batch_request(requests)
        
        # Verify error details
        assert exc_info.value.code == -32600
        assert exc_info.value.message == "Invalid Request"
        
        # Test with ignore_errors=True
        results = client.batch_request(requests, ignore_errors=True)
        assert len(results) == 2
        assert results[0] == {"value": "result1"}
        assert isinstance(results[1], RpcError)
        assert results[1].code == -32600
    
    @patch('rooch.rpc.client.httpx.Client.post')
    def test_request_with_different_parameter_types(self, mock_post):
        """Test JSON-RPC request with different parameter types"""
        # Setup mock response
        mock_response = MagicMock()
        mock_response.status_code = 200
        mock_response.json.return_value = {
            "jsonrpc": "2.0",
            "id": 1,
            "result": {"success": True}
        }
        mock_post.return_value = mock_response
        
        # Create client
        client = JsonRpcClient()
        
        # Test with number parameters
        client.request("test_method", 123)
        args, kwargs = mock_post.call_args
        assert kwargs["json"]["params"] == 123
        
        # Test with array parameters
        client.request("test_method", [1, 2, 3])
        args, kwargs = mock_post.call_args
        assert kwargs["json"]["params"] == [1, 2, 3]
        
        # Test with nested object parameters
        complex_params = {
            "obj": {"nested": {"value": 42}},
            "array": [1, {"key": "value"}, 3],
            "null": None,
            "bool": True
        }
        client.request("test_method", complex_params)
        args, kwargs = mock_post.call_args
        assert kwargs["json"]["params"] == complex_params
    
    @patch('rooch.rpc.client.httpx.Client.post')
    def test_request_with_empty_params(self, mock_post):
        """Test JSON-RPC request with empty parameters"""
        # Setup mock response
        mock_response = MagicMock()
        mock_response.status_code = 200
        mock_response.json.return_value = {
            "jsonrpc": "2.0",
            "id": 1,
            "result": {"success": True}
        }
        mock_post.return_value = mock_response
        
        # Create client
        client = JsonRpcClient()
        
        # Test with None parameters
        client.request("test_method", None)
        args, kwargs = mock_post.call_args
        assert "params" not in kwargs["json"]
        
        # Test with empty dict parameters
        client.request("test_method", {})
        args, kwargs = mock_post.call_args
        assert kwargs["json"]["params"] == {}
        
        # Test with empty list parameters
        client.request("test_method", [])
        args, kwargs = mock_post.call_args
        assert kwargs["json"]["params"] == []
    
    @patch('rooch.rpc.client.httpx.Client')
    def test_client_with_timeout(self, mock_client_class):
        """Test client with custom timeout"""
        # Setup mock
        mock_client = MagicMock()
        mock_client_class.return_value = mock_client
        mock_response = MagicMock()
        mock_response.status_code = 200
        mock_response.json.return_value = {
            "jsonrpc": "2.0",
            "id": 1,
            "result": {"success": True}
        }
        mock_client.post.return_value = mock_response
        
        # Create client with custom timeout
        client = JsonRpcClient(timeout=30)
        
        # Make request and check timeout was passed
        client.request("test_method", {"param": "value"})
        
        # Check that the client was created with the correct timeout
        mock_client_class.assert_called_once()
        args, kwargs = mock_client_class.call_args
        assert kwargs["timeout"] == 30
    
    @patch('rooch.rpc.client.httpx.Client')
    def test_client_with_headers(self, mock_client_class):
        """Test client with custom headers"""
        # Setup mock client instance behavior (post method)
        mock_client_instance = MagicMock()
        mock_response = MagicMock()
        mock_response.status_code = 200
        mock_response.json.return_value = {
            "jsonrpc": "2.0",
            "id": 1,
            "result": {"success": True}
        }
        mock_client_instance.post.return_value = mock_response
        # Make the mock class return our instance
        mock_client_class.return_value = mock_client_instance

        # Create client with custom headers
        custom_headers = {
            "Authorization": "Bearer token123",
            "X-Custom-Header": "custom_value"
        }
        # Also specify a non-default timeout to check both params are passed
        custom_timeout = 60.0 
        client = JsonRpcClient(headers=custom_headers, timeout=custom_timeout)

        # Check if httpx.Client was initialized correctly
        # httpx default timeout might be different, so capture it if needed
        # For now, let's assume we want to verify the passed headers and timeout
        mock_client_class.assert_called_once_with(headers=custom_headers, timeout=custom_timeout)

        # Make a request to ensure the post call happens
        client.request("test_method", {"param": "value"})

        # Check that the post request was made (we don't need to check headers here anymore)
        mock_client_instance.post.assert_called_once()
        # args, kwargs = mock_client_instance.post.call_args
        # assert "headers" not in kwargs # Or check specific headers if needed via mock_client_instance.headers
        # Example: Check if the underlying client has the headers set (requires mock setup)
        # assert mock_client_instance.headers["Authorization"] == "Bearer token123"
    
    @patch('rooch.rpc.client.httpx.Client.post')
    def test_malformed_json_response(self, mock_post):
        """Test handling of malformed JSON responses"""
        # Setup mock response that raises when json() is called
        mock_response = MagicMock()
        mock_response.status_code = 200
        mock_response.json.side_effect = json.JSONDecodeError("Malformed JSON", "", 0)
        mock_post.return_value = mock_response
        
        # Create client and verify error is raised
        client = JsonRpcClient()
        with pytest.raises(Exception) as exc_info:
            client.request("test_method", {"param": "value"})
        
        assert "Malformed JSON" in str(exc_info.value)