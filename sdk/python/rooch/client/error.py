#!/usr/bin/env python3
# Copyright (c) Rooch Network
# SPDX-License-Identifier: Apache-2.0

from typing import Any, Optional


class RoochError(Exception):
    """Base class for all Rooch SDK errors"""
    pass


class RoochTransportError(RoochError):
    """Error from the transport layer"""
    
    def __init__(
        self, 
        message: str, 
        code: Optional[int] = None, 
        data: Optional[Any] = None
    ):
        """
        Args:
            message: Error message
            code: Error code (if available)
            data: Additional error data (if available)
        """
        super().__init__(message)
        self.code = code
        self.data = data


class RoochClientError(RoochError):
    """Error from the client layer"""
    pass


class RoochSubscriptionError(RoochError):
    """Error related to subscriptions"""
    pass


class RoochDeserializationError(RoochError):
    """Error while deserializing data from the blockchain"""
    pass


class RoochSerializationError(RoochError):
    """Error while serializing data for the blockchain"""
    pass