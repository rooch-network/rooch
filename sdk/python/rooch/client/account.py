#!/usr/bin/env python3
# Copyright (c) Rooch Network
# SPDX-License-Identifier: Apache-2.0

from typing import Any, Dict, List, Optional, Union

from ..transport import RoochTransport


class AccountClient:
    """Client for Rooch account operations"""
    
    def __init__(self, transport: RoochTransport):
        """Initialize with a transport
        
        Args:
            transport: Transport for communicating with the Rooch node
        """
        self._transport = transport
    
    async def get_account(self, address: str) -> Dict[str, Any]:
        """Get account information
        
        Args:
            address: Account address
            
        Returns:
            Account information
        """
        return await self._transport.request("rooch_getAccount", [address])
    
    async def get_account_sequence_number(self, address: str) -> int:
        """Get account sequence number
        
        Args:
            address: Account address
            
        Returns:
            Sequence number
        """
        result = await self._transport.request("rooch_getAccountSequenceNumber", [address])
        # The result can be a string or int depending on the node
        return int(result) if isinstance(result, str) else result
    
    async def get_balance(self, address: str, coin_type: Optional[str] = None) -> Dict[str, Any]:
        """Get account balance
        
        Args:
            address: Account address
            coin_type: Optional coin type (e.g., "0x1::coin::ROOCH")
            
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