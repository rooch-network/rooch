#!/usr/bin/env python3
# Copyright (c) Rooch Network
# SPDX-License-Identifier: Apache-2.0

from typing import Any, Dict, List, Optional, Union

from ..transport import RoochTransport
from ..transactions.serializer import TxSerializer
from ..address.rooch import RoochAddress
from ..bcs.serializer import BcsSerializer
from ..utils.hex import to_hex
from rooch.transport import RoochTransportError

# Constants for view function
FUNC_SEQUENCE_NUMBER = "0x2::account::sequence_number"

class AccountClient:
    """Client for Rooch account operations"""
    
    def __init__(self, transport: RoochTransport):
        """Initialize with a transport
        
        Args:
            transport: Transport for communicating with the Rooch node
        """
        self._transport = transport
    
    async def get_account(self, address: str) -> Optional[Dict[str, Any]]:
        """Get account object state by address.

        Note: Rooch addresses are derived from public keys, but the core account
        object ID might be different. We assume here the address string
        can also function as the ObjectID for the account object.
        This might need adjustment based on how account objects are identified.

        Args:
            address: Account address (assumed to be the ObjectID)

        Returns:
            Account object state dictionary or None if not found.
        """
        # Use getObjectStates which expects a list of ObjectIDs
        # Assuming the address string corresponds to the ObjectID of the account
        object_id = address

        results = await self._transport.request(
            "rooch_getObjectStates",
            [object_id] # Pass object_id directly in a list
            # Optionally add state options if needed: , [{"decode": True, "showDisplay": True}]
        )

        # Results is Vec<Option<ObjectStateView>>
        if results and len(results) > 0 and results[0] is not None:
            account_state = results[0]
            # Add the address back for consistency with previous tests if needed?
            # account_state["address"] = address
            return account_state
        else:
            return None
    
    async def get_account_sequence_number(self, address: str) -> int:
        """Get account sequence number using a view function call. Returns 0 if the account does not exist."""

        # Convert address to the format expected by the view function (raw address bytes as hex)
        try:
            from ..address.rooch import RoochAddress
            from ..utils.hex import to_hex
            
            # Convert address to RoochAddress and get raw bytes
            rooch_address = RoochAddress.from_str(address)
            address_arg_hex = rooch_address.to_hex_full()
        except Exception as e:
            raise ValueError(f"Invalid address format {address}: {e}") from e

        # Construct the function call as a dictionary
        function_call = {
            "function_id": FUNC_SEQUENCE_NUMBER,
            "ty_args": [],
            "args": [
                # Pass the raw address bytes as hex string (not the full address)
                address_arg_hex
            ]
        }

        try:
            # Call the view function with the dictionary payload
            result = await self._transport.request(
                "rooch_executeViewFunction",
                [function_call, {"decode": True}] # Pass the dictionary directly inside a list
            )

            # Check for VM errors first (e.g., account not found)
            if result and isinstance(result, dict):
                vm_status = result.get("vm_status")
                if isinstance(vm_status, dict):
                    vm_error = vm_status.get("Error") # Check specific Move VM error codes
                    # TODO: Use defined constants for VM error codes if available
                    if vm_error == '1091': # 1091 typically means ACCOUNT_NOT_FOUND in SequenceNumber context
                        return 0
                    elif vm_error: # If there's any other VM error
                        raise ValueError(f"View function failed with VM Error: {vm_status}")
            
            # If no VM error, proceed to extract the sequence number from the result
            if result and result.get("return_values") and len(result["return_values"]) > 0:
                return_value = result["return_values"][0]
                if return_value and isinstance(return_value, dict):
                    # Check if decoded_value exists and get its value
                    if "decoded_value" in return_value:
                        decoded_value = return_value["decoded_value"]
                        # decoded_value can be either a dict with "value" key or a direct value
                        if isinstance(decoded_value, dict) and "value" in decoded_value:
                            seq_num_str = decoded_value["value"]
                        else:
                            # decoded_value is the direct value
                            seq_num_str = str(decoded_value)
                        
                        try:
                            return int(seq_num_str)
                        except ValueError:
                            raise ValueError(f"Could not parse sequence number as integer: {seq_num_str}")
                    else:
                        raise ValueError(f"No 'decoded_value' in return_value: {return_value}")
                else:
                    raise ValueError(f"Unexpected 'return_value' format: {return_value}")

            # If parsing fails or result structure is unexpected after ruling out known VM errors
            raise ValueError(f"Could not extract sequence number from view function result: {result}")

        except RoochTransportError as e:
            # This catches transport-level errors or errors re-raised from the try block
            # Check if the transport error itself indicates account not found (might be redundant now)
            if "Account not found" in e.message or "VMError with status ACCOUNT_NOT_FOUND" in e.message:
                 return 0
            else:
                # Re-raise other transport errors
                raise e
    
    async def get_balance(self, address: str, coin_type: Optional[str] = None) -> Dict[str, Any]:
        """Get account balance
        
        Args:
            address: Account address
            coin_type: Optional coin type (e.g., "0x3::gas_coin::RGas")
            
        Returns:
            Balance information
        """
        if coin_type:
            return await self._transport.request("rooch_getBalance", [address, coin_type])
        else:
            return await self._transport.request("rooch_getBalance", [address])
    
    async def get_balances(self, address: str) -> List[Dict[str, Any]]:
        """Get all balances for an account
        
        Args:
            address: Account address
            
        Returns:
            List of balance information for different coin types
        """
        return await self._transport.request("rooch_getBalances", [address])
    
    async def get_resource(
        self, 
        address: str, 
        resource_type: str,
        decode: bool = True
    ) -> Dict[str, Any]:
        """Get a resource from an account
        
        Args:
            address: Account address
            resource_type: Resource type
            decode: Whether to decode the resource data
            
        Returns:
            Resource data
        """
        return await self._transport.request("rooch_getResource", [address, resource_type, decode])
    
    async def get_resources(
        self, 
        address: str,
        decode: bool = True
    ) -> List[Dict[str, Any]]:
        """Get all resources from an account
        
        Args:
            address: Account address
            decode: Whether to decode the resource data
            
        Returns:
            List of resource data
        """
        return await self._transport.request("rooch_getResources", [address, decode])
    
    async def get_resource_by_index(
        self, 
        address: str, 
        resource_index: str,
        decode: bool = True
    ) -> Dict[str, Any]:
        """Get a resource from an account by index
        
        Args:
            address: Account address
            resource_index: Resource index
            decode: Whether to decode the resource data
            
        Returns:
            Resource data
        """
        return await self._transport.request("rooch_getResourceByIndex", [address, resource_index, decode])
    
    async def get_module(
        self, 
        address: str, 
        module_name: str,
        decode: bool = True
    ) -> Dict[str, Any]:
        """Get a module from an account
        
        Args:
            address: Account address
            module_name: Module name
            decode: Whether to decode the module bytecode
            
        Returns:
            Module data
        """
        return await self._transport.request("rooch_getModule", [address, module_name, decode])
    
    async def get_modules(
        self, 
        address: str,
        decode: bool = True
    ) -> List[Dict[str, Any]]:
        """Get all modules from an account
        
        Args:
            address: Account address
            decode: Whether to decode the module bytecode
            
        Returns:
            List of module data
        """
        return await self._transport.request("rooch_getModules", [address, decode])
    
    async def get_module_by_index(
        self, 
        address: str, 
        module_index: str,
        decode: bool = True
    ) -> Dict[str, Any]:
        """Get a module from an account by index
        
        Args:
            address: Account address
            module_index: Module index
            decode: Whether to decode the module bytecode
            
        Returns:
            Module data
        """
        return await self._transport.request("rooch_getModuleByIndex", [address, module_index, decode])