#!/usr/bin/env python3
# Copyright (c) Rooch Network
# SPDX-License-Identifier: Apache-2.0

from typing import List, Dict, Any, Optional, Callable, TypeVar, AsyncGenerator
from rooch.client import RoochClient
from .pagination import paginate_all, iterate_pages

T = TypeVar('T')

class RoochClientHelpers:
    """Helper class providing pagination utilities for RoochClient"""
    
    def __init__(self, client: RoochClient):
        self.client = client
        
    async def get_all_states(self, filter_type: str, filter_value: str) -> List[Dict[str, Any]]:
        """Get all states matching the filter, handling pagination automatically
        
        Args:
            filter_type: The type of filter to apply
            filter_value: The value to filter by
            
        Returns:
            List of all matching states
        """
        async def fetch_page(cursor: int, limit: int):
            return await self.client.get_states(filter_type, filter_value, cursor=cursor, limit=limit)
            
        def extract_items(page: Dict[str, Any]) -> List[Dict[str, Any]]:
            return page.get("data", {}).get("states", [])
            
        def extract_next_cursor(page: Dict[str, Any]) -> Optional[int]:
            pagination = page.get("data", {}).get("pagination", {})
            if pagination.get("has_next_page", False):
                return pagination.get("next_cursor")
            return None
            
        return await paginate_all(fetch_page, extract_items, extract_next_cursor)
        
    async def get_all_states_by_prefix(self, address: str, prefix: str) -> List[Dict[str, Any]]:
        """Get all states with prefix, handling pagination automatically
        
        Args:
            address: The address to query
            prefix: The prefix to filter by
            
        Returns:
            List of all matching states
        """
        async def fetch_page(cursor: int, limit: int):
            return await self.client.get_states_by_prefix(address, prefix, cursor=cursor, limit=limit)
            
        def extract_items(page: Dict[str, Any]) -> List[Dict[str, Any]]:
            return page.get("data", {}).get("states", [])
            
        def extract_next_cursor(page: Dict[str, Any]) -> Optional[int]:
            pagination = page.get("data", {}).get("pagination", {})
            if pagination.get("has_next_page", False):
                return pagination.get("next_cursor")
            return None
            
        return await paginate_all(fetch_page, extract_items, extract_next_cursor)
        
    async def iterate_states(self, filter_type: str, filter_value: str, batch_size: int = 25) -> AsyncGenerator[Dict[str, Any], None]:
        """Iterate through all states matching the filter
        
        Args:
            filter_type: The type of filter to apply
            filter_value: The value to filter by
            batch_size: Number of items to fetch per page
            
        Yields:
            Individual state items
        """
        async def fetch_page(cursor: int, limit: int):
            return await self.client.get_states(filter_type, filter_value, cursor=cursor, limit=limit)
            
        def extract_items(page: Dict[str, Any]) -> List[Dict[str, Any]]:
            return page.get("data", {}).get("states", [])
            
        def extract_next_cursor(page: Dict[str, Any]) -> Optional[int]:
            pagination = page.get("data", {}).get("pagination", {})
            if pagination.get("has_next_page", False):
                return pagination.get("next_cursor")
            return None
            
        async for item in iterate_pages(fetch_page, extract_items, extract_next_cursor, batch_size):
            yield item
            
    async def iterate_states_by_prefix(self, address: str, prefix: str, batch_size: int = 25) -> AsyncGenerator[Dict[str, Any], None]:
        """Iterate through all states with prefix
        
        Args:
            address: The address to query
            prefix: The prefix to filter by
            batch_size: Number of items to fetch per page
            
        Yields:
            Individual state items
        """
        async def fetch_page(cursor: int, limit: int):
            return await self.client.get_states_by_prefix(address, prefix, cursor=cursor, limit=limit)
            
        def extract_items(page: Dict[str, Any]) -> List[Dict[str, Any]]:
            return page.get("data", {}).get("states", [])
            
        def extract_next_cursor(page: Dict[str, Any]) -> Optional[int]:
            pagination = page.get("data", {}).get("pagination", {})
            if pagination.get("has_next_page", False):
                return pagination.get("next_cursor")
            return None
            
        async for item in iterate_pages(fetch_page, extract_items, extract_next_cursor, batch_size):
            yield item