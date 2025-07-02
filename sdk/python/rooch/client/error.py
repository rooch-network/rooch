#!/usr/bin/env python3
# Copyright (c) Rooch Network
# SPDX-License-Identifier: Apache-2.0

"""
Enhanced error handling for the Rooch Python SDK.
This module provides a hierarchy of exception classes for different types of errors.
"""

from typing import Any, Dict, Optional


class RoochError(Exception):
    """Base exception class for all Rooch SDK errors"""
    
    def __init__(self, message: str, code: Optional[int] = None, data: Any = None):
        """Initialize with message and optional code and data
        
        Args:
            message: Error message
            code: Error code (if available)
            data: Additional error data (if available)
        """
        self.message = message
        self.code = code
        self.data = data
        super().__init__(self.formatted_message())
    
    def formatted_message(self) -> str:
        """Format error message with code if available
        
        Returns:
            Formatted error message
        """
        if self.code is not None:
            return f"Error {self.code}: {self.message}"
        return self.message

    def __str__(self) -> str:
        """String representation
        
        Returns:
            String representation of error
        """
        return self.formatted_message()


class RoochTransportError(RoochError):
    """Error related to transport layer (HTTP/WebSocket)"""
    
    def __init__(self, message: str, code: Optional[int] = None, data: Any = None, 
                 url: Optional[str] = None):
        """Initialize with message, code, data, and URL
        
        Args:
            message: Error message
            code: Error code (if available)
            data: Additional error data (if available)
            url: URL that caused the error (if applicable)
        """
        self.url = url
        full_message = message
        if url:
            full_message = f"{message} (URL: {url})"
        super().__init__(full_message, code, data)


class RoochTimeoutError(RoochTransportError):
    """Timeout error for Rooch requests"""
    
    def __init__(self, message: str, timeout_ms: Optional[int] = None, 
                 url: Optional[str] = None, request_method: Optional[str] = None):
        """Initialize with message, timeout, URL, and request method
        
        Args:
            message: Error message
            timeout_ms: Timeout in milliseconds
            url: URL that timed out
            request_method: The RPC method that was called
        """
        self.timeout_ms = timeout_ms
        self.request_method = request_method
        
        full_message = message
        if timeout_ms:
            full_message = f"{message} (Timeout: {timeout_ms}ms)"
        if request_method:
            full_message = f"{full_message}, Method: {request_method}"
            
        super().__init__(full_message, url=url)


class RoochConnectionError(RoochTransportError):
    """Connection error for Rooch requests"""
    pass


class RoochClientError(RoochError):
    """Error from the client layer"""
    
    def __init__(self, message: str, method: Optional[str] = None, 
                 params: Optional[Any] = None, code: Optional[int] = None):
        """Initialize with message, method, parameters, and code
        
        Args:
            message: Error message
            method: Method that caused the error
            params: Parameters that caused the error
            code: Error code
        """
        self.method = method
        self.params = params
        
        full_message = message
        details = []
        
        if method:
            details.append(f"Method: {method}")
            
        if details:
            detail_str = ", ".join(details)
            full_message = f"{message} ({detail_str})"
            
        super().__init__(full_message, code)


class RoochSubscriptionError(RoochError):
    """Error related to WebSocket subscriptions"""
    
    def __init__(self, message: str, subscription_id: Optional[str] = None, 
                 method: Optional[str] = None):
        """Initialize with message, subscription ID, and method
        
        Args:
            message: Error message
            subscription_id: Subscription ID (if applicable)
            method: Subscription method (if applicable)
        """
        self.subscription_id = subscription_id
        self.method = method
        
        full_message = message
        details = []
        
        if subscription_id:
            details.append(f"Subscription ID: {subscription_id}")
        if method:
            details.append(f"Method: {method}")
            
        if details:
            detail_str = ", ".join(details)
            full_message = f"{message} ({detail_str})"
            
        super().__init__(full_message)


class RoochRpcError(RoochError):
    """Error returned by Rooch RPC"""
    
    def __init__(self, message: str, code: Optional[int] = None, data: Any = None):
        """Initialize with message, code, and data
        
        Args:
            message: Error message
            code: Error code
            data: Additional error data
        """
        super().__init__(message, code, data)


class RoochTxExecutionError(RoochError):
    """Error during transaction execution"""
    
    def __init__(self, message: str, tx_hash: Optional[str] = None, 
                 vm_status: Optional[str] = None, details: Optional[Dict[str, Any]] = None):
        """Initialize with message, transaction hash, VM status, and details
        
        Args:
            message: Error message
            tx_hash: Transaction hash
            vm_status: VM status (if available)
            details: Additional details about the transaction
        """
        self.tx_hash = tx_hash
        self.vm_status = vm_status
        self.details = details
        
        full_message = message
        if tx_hash:
            full_message = f"{message} (TX: {tx_hash})"
        if vm_status:
            full_message = f"{full_message}, VM Status: {vm_status}"
            
        super().__init__(full_message)


class RoochMoveError(RoochError):
    """Error related to Move language"""
    
    def __init__(self, message: str, function_id: Optional[str] = None, 
                 module_id: Optional[str] = None, code: Optional[int] = None):
        """Initialize with message, function ID, module ID, and code
        
        Args:
            message: Error message
            function_id: Function ID (if applicable)
            module_id: Module ID (if applicable)
            code: Error code (if available)
        """
        self.function_id = function_id
        self.module_id = module_id
        
        full_message = message
        details = []
        
        if function_id:
            details.append(f"Function: {function_id}")
        elif module_id:
            details.append(f"Module: {module_id}")
            
        if details:
            detail_str = ", ".join(details)
            full_message = f"{message} ({detail_str})"
            
        super().__init__(full_message, code)


class RoochAddressError(RoochError):
    """Error related to address handling"""
    
    def __init__(self, message: str, address: Optional[str] = None, 
                 address_type: Optional[str] = None):
        """Initialize with message, address, and address type
        
        Args:
            message: Error message
            address: The problematic address
            address_type: Type of address (e.g., "rooch", "bitcoin")
        """
        self.address = address
        self.address_type = address_type
        
        full_message = message
        if address:
            full_message = f"{message} (Address: {address}"
            if address_type:
                full_message = f"{full_message}, Type: {address_type}"
            full_message = f"{full_message})"
            
        super().__init__(full_message)


class RoochDeserializationError(RoochError):
    """Error while deserializing data from the blockchain"""
    
    def __init__(self, message: str, object_type: Optional[str] = None, 
                 field: Optional[str] = None, data: Any = None):
        """Initialize with message, object type, field, and data
        
        Args:
            message: Error message
            object_type: Type of object being deserialized
            field: Field that caused the error (if applicable)
            data: The data that failed to deserialize
        """
        self.object_type = object_type
        self.field = field
        
        full_message = message
        details = []
        
        if object_type:
            details.append(f"Type: {object_type}")
        if field:
            details.append(f"Field: {field}")
            
        if details:
            detail_str = ", ".join(details)
            full_message = f"{message} ({detail_str})"
            
        super().__init__(full_message, data=data)


class RoochSerializationError(RoochError):
    """Error while serializing data for the blockchain"""
    
    def __init__(self, message: str, object_type: Optional[str] = None, 
                 field: Optional[str] = None, value: Any = None):
        """Initialize with message, object type, field, and value
        
        Args:
            message: Error message
            object_type: Type of object being serialized
            field: Field that caused the error (if applicable)
            value: The value that failed to serialize
        """
        self.object_type = object_type
        self.field = field
        self.value = value
        
        full_message = message
        details = []
        
        if object_type:
            details.append(f"Type: {object_type}")
        if field:
            details.append(f"Field: {field}")
            
        if details:
            detail_str = ", ".join(details)
            full_message = f"{message} ({detail_str})"
            
        super().__init__(full_message)


class RoochSessionError(RoochError):
    """Error related to session management"""
    
    def __init__(self, message: str, session_id: Optional[str] = None, 
                 app_name: Optional[str] = None):
        """Initialize with message, session ID, and app name
        
        Args:
            message: Error message
            session_id: Session ID (if applicable)
            app_name: Application name (if applicable)
        """
        self.session_id = session_id
        self.app_name = app_name
        
        full_message = message
        details = []
        
        if session_id:
            details.append(f"Session ID: {session_id}")
        if app_name:
            details.append(f"App: {app_name}")
            
        if details:
            detail_str = ", ".join(details)
            full_message = f"{message} ({detail_str})"
            
        super().__init__(full_message)


class RoochConfigError(RoochError):
    """Error related to configuration"""
    pass


class RoochValidationError(RoochError):
    """Error during input validation"""
    
    def __init__(self, message: str, field: Optional[str] = None, 
                 value: Optional[Any] = None, expected: Optional[str] = None):
        """Initialize with message, field, value, and expected format
        
        Args:
            message: Error message
            field: Field that failed validation
            value: Value that failed validation
            expected: Expected format or value
        """
        self.field = field
        self.value = value
        self.expected = expected
        
        full_message = message
        details = []
        
        if field:
            details.append(f"Field: {field}")
        if value is not None:
            # Use str() to safely represent the value
            details.append(f"Value: {str(value)}")
        if expected:
            details.append(f"Expected: {expected}")
            
        if details:
            detail_str = ", ".join(details)
            full_message = f"{message} ({detail_str})"
            
        super().__init__(full_message)