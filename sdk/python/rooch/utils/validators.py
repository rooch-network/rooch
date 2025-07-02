#!/usr/bin/env python3
# Copyright (c) Rooch Network
# SPDX-License-Identifier: Apache-2.0

"""
Validators for Rooch SDK inputs.
Provides utility functions to validate addresses, function IDs, type arguments, etc.
"""

import re
from typing import Any, Dict, List, Optional, Union

from ..client.error import RoochValidationError


def validate_address(address: str, allow_empty: bool = False) -> str:
    """Validate a Rooch address
    
    Args:
        address: Address to validate
        allow_empty: Whether to allow empty address
        
    Returns:
        Normalized address string
        
    Raises:
        RoochValidationError: If address is invalid
    """
    if not address and allow_empty:
        return address
        
    if not address or not isinstance(address, str):
        raise RoochValidationError(
            "Invalid address",
            field="address",
            value=address,
            expected="0x-prefixed hex string"
        )
    
    # Normalize to lowercase with 0x prefix
    normalized = address.lower()
    if not normalized.startswith("0x"):
        normalized = f"0x{normalized}"
    
    # Check that it's a valid hex string of appropriate length
    hex_pattern = r"^0x[0-9a-f]+$"
    if not re.match(hex_pattern, normalized):
        raise RoochValidationError(
            "Address must be a valid hex string",
            field="address",
            value=address,
            expected="0x-prefixed hex string"
        )
    
    return normalized


def validate_function_id(function_id: str) -> str:
    """Validate a function ID
    
    Args:
        function_id: Function ID to validate (format: 0xADDRESS::module::function)
        
    Returns:
        Normalized function ID string
        
    Raises:
        RoochValidationError: If function ID is invalid
    """
    if not function_id or not isinstance(function_id, str):
        raise RoochValidationError(
            "Invalid function ID",
            field="function_id",
            value=function_id,
            expected="0xADDRESS::module::function"
        )
    
    # Check format: 0xADDRESS::module::function
    pattern = r"^(0x[0-9a-fA-F]+)::([\w\d_]+)::([\w\d_]+)$"
    match = re.match(pattern, function_id)
    if not match:
        raise RoochValidationError(
            "Function ID must be in format 0xADDRESS::module::function",
            field="function_id",
            value=function_id,
            expected="0xADDRESS::module::function"
        )
    
    # Normalize to lowercase address
    address, module, function = match.groups()
    normalized_address = address.lower()
    
    return f"{normalized_address}::{module}::{function}"


def validate_module_id(module_id: str) -> str:
    """Validate a module ID
    
    Args:
        module_id: Module ID to validate (format: 0xADDRESS::module)
        
    Returns:
        Normalized module ID string
        
    Raises:
        RoochValidationError: If module ID is invalid
    """
    if not module_id or not isinstance(module_id, str):
        raise RoochValidationError(
            "Invalid module ID",
            field="module_id",
            value=module_id,
            expected="0xADDRESS::module"
        )
    
    # Check format: 0xADDRESS::module
    pattern = r"^(0x[0-9a-fA-F]+)::([\w\d_]+)$"
    match = re.match(pattern, module_id)
    if not match:
        raise RoochValidationError(
            "Module ID must be in format 0xADDRESS::module",
            field="module_id",
            value=module_id,
            expected="0xADDRESS::module"
        )
    
    # Normalize to lowercase address
    address, module = match.groups()
    normalized_address = address.lower()
    
    return f"{normalized_address}::{module}"


def validate_type_arg(type_arg: str) -> str:
    """Validate a type argument
    
    Args:
        type_arg: Type argument to validate
        
    Returns:
        Normalized type argument string
        
    Raises:
        RoochValidationError: If type argument is invalid
    """
    if not type_arg or not isinstance(type_arg, str):
        raise RoochValidationError(
            "Invalid type argument",
            field="type_arg",
            value=type_arg,
            expected="Valid Move type argument (e.g., 0x1::coin::Coin<0x3::gas_coin::GasCoin>)"
        )
    
    # Simplified validation for now
    # TODO: Implement more comprehensive type validation
    
    return type_arg


def validate_type_args(type_args: List[str]) -> List[str]:
    """Validate a list of type arguments
    
    Args:
        type_args: List of type arguments to validate
        
    Returns:
        List of normalized type arguments
        
    Raises:
        RoochValidationError: If type arguments are invalid
    """
    if type_args is None:
        return []
    
    if not isinstance(type_args, list):
        raise RoochValidationError(
            "Type arguments must be a list",
            field="type_args",
            value=type_args,
            expected="List of type arguments"
        )
    
    return [validate_type_arg(arg) for arg in type_args]


def validate_gas_amount(gas_amount: Union[int, str]) -> int:
    """Validate gas amount
    
    Args:
        gas_amount: Gas amount to validate
        
    Returns:
        Gas amount as integer
        
    Raises:
        RoochValidationError: If gas amount is invalid
    """
    try:
        if isinstance(gas_amount, str):
            gas = int(gas_amount)
        else:
            gas = gas_amount
    except (ValueError, TypeError):
        raise RoochValidationError(
            "Gas amount must be a valid integer",
            field="gas_amount",
            value=gas_amount,
            expected="Integer value"
        )
    
    if gas < 0:
        raise RoochValidationError(
            "Gas amount must be non-negative",
            field="gas_amount",
            value=gas_amount,
            expected="Non-negative integer"
        )
    
    return gas


def validate_sequence_number(sequence_number: Union[int, str]) -> int:
    """Validate sequence number
    
    Args:
        sequence_number: Sequence number to validate
        
    Returns:
        Sequence number as integer
        
    Raises:
        RoochValidationError: If sequence number is invalid
    """
    try:
        if isinstance(sequence_number, str):
            seq_num = int(sequence_number)
        else:
            seq_num = sequence_number
    except (ValueError, TypeError):
        raise RoochValidationError(
            "Sequence number must be a valid integer",
            field="sequence_number",
            value=sequence_number,
            expected="Integer value"
        )
    
    if seq_num < 0:
        raise RoochValidationError(
            "Sequence number must be non-negative",
            field="sequence_number",
            value=sequence_number,
            expected="Non-negative integer"
        )
    
    return seq_num


def validate_chain_id(chain_id: Union[int, str]) -> int:
    """Validate chain ID
    
    Args:
        chain_id: Chain ID to validate
        
    Returns:
        Chain ID as integer
        
    Raises:
        RoochValidationError: If chain ID is invalid
    """
    try:
        if isinstance(chain_id, str):
            id_val = int(chain_id)
        else:
            id_val = chain_id
    except (ValueError, TypeError):
        raise RoochValidationError(
            "Chain ID must be a valid integer",
            field="chain_id",
            value=chain_id,
            expected="Integer value"
        )
    
    if id_val < 0:
        raise RoochValidationError(
            "Chain ID must be non-negative",
            field="chain_id",
            value=chain_id,
            expected="Non-negative integer"
        )
    
    return id_val


def validate_hex_string(hex_str: str, field_name: str, allow_empty: bool = False) -> str:
    """Validate a hex string
    
    Args:
        hex_str: Hex string to validate
        field_name: Name of the field for error reporting
        allow_empty: Whether to allow empty string
        
    Returns:
        Normalized hex string with 0x prefix
        
    Raises:
        RoochValidationError: If hex string is invalid
    """
    if not hex_str and allow_empty:
        return ""
        
    if not hex_str or not isinstance(hex_str, str):
        raise RoochValidationError(
            f"Invalid {field_name}",
            field=field_name,
            value=hex_str,
            expected="0x-prefixed hex string"
        )
    
    # Normalize to lowercase with 0x prefix
    normalized = hex_str.lower()
    if not normalized.startswith("0x"):
        normalized = f"0x{normalized}"
    
    # Check that it's a valid hex string
    hex_pattern = r"^0x[0-9a-f]+$"
    if not re.match(hex_pattern, normalized):
        raise RoochValidationError(
            f"{field_name} must be a valid hex string",
            field=field_name,
            value=hex_str,
            expected="0x-prefixed hex string"
        )
    
    return normalized
