#!/usr/bin/env python3
# Copyright (c) Rooch Network
# SPDX-License-Identifier: Apache-2.0

"""
BCS module exports for Move function argument encoding.

This module provides the new type-safe argument encoding system
that separates parameter serialization from transaction serialization.
"""

# Core argument encoding classes
from .args import Args, ArgType, infer_and_encode

# Function call builders
from .function_builder import (
    MoveFunctionBuilder,
    transfer_coin,
    faucet_claim,
    create_u256_args,
    create_address_args,
    create_mixed_args
)

# Original serialization classes (for internal use)
from .serializer import (
    BcsSerializer,
    BcsDeserializer,
    Serializable,
    Deserializable,
    BcsSerializationError,
    BcsDeserializationError
)

__all__ = [
    # New argument encoding API (recommended)
    'Args',
    'ArgType',
    'MoveFunctionBuilder',
    'transfer_coin',
    'faucet_claim',
    'create_u256_args',
    'create_address_args',
    
    # Type inference (use with caution)
    'infer_and_encode',
    'create_mixed_args',
    
    # Low-level serialization (internal use)
    'BcsSerializer',
    'BcsDeserializer',
    'Serializable',
    'Deserializable',
    'BcsSerializationError',
    'BcsDeserializationError',
]
