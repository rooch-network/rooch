#!/usr/bin/env python3
# Copyright (c) Rooch Network
# SPDX-License-Identifier: Apache-2.0

from typing import Any, Dict, List, Optional, Callable, AsyncGenerator, TypeVar, Generic

T = TypeVar('T')

async def paginate_all(
    fetch_page_function: Callable[[int, int], Any],
    extract_items_function: Callable[[Any], List[T]],
    extract_next_cursor_function: Callable[[Any], Optional[int]],
    batch_size: int = 25,
    max_items: Optional[int] = None
) -> List[T]:
    """Fetch all pages of paginated results
    
    Args:
        fetch_page_function: Function that fetches a page given cursor and limit
        extract_items_function: Function that extracts items from a page response
        extract_next_cursor_function: Function that extracts the next cursor from a page response
        batch_size: Number of items to fetch per page
        max_items: Maximum number of items to fetch (None for all)
        
    Returns:
        List of all items across pages
    """
    all_items = []
    cursor = 0
    
    while True:
        # Determine how many items to fetch in this batch
        current_batch_size = batch_size
        if max_items is not None:
            remaining = max_items - len(all_items)
            if remaining <= 0:
                break
            current_batch_size = min(batch_size, remaining)
            
        # Fetch the page
        page = await fetch_page_function(cursor, current_batch_size)
        
        # Extract items from the page
        items = extract_items_function(page)
        all_items.extend(items)
        
        # Extract the next cursor
        next_cursor = extract_next_cursor_function(page)
        
        # Break if no more pages
        if next_cursor is None or next_cursor <= cursor or len(items) == 0:
            break
            
        cursor = next_cursor
    
    return all_items

async def iterate_pages(
    fetch_page_function: Callable[[int, int], Any],
    extract_items_function: Callable[[Any], List[T]],
    extract_next_cursor_function: Callable[[Any], Optional[int]],
    batch_size: int = 25
) -> AsyncGenerator[T, None]:
    """Iterate through all pages of paginated results
    
    Args:
        fetch_page_function: Function that fetches a page given cursor and limit
        extract_items_function: Function that extracts items from a page response
        extract_next_cursor_function: Function that extracts the next cursor from a page response
        batch_size: Number of items to fetch per page
        
    Yields:
        Individual items from all pages
    """
    cursor = 0
    
    while True:
        # Fetch the page
        page = await fetch_page_function(cursor, batch_size)
        
        # Extract items from the page
        items = extract_items_function(page)
        
        # Yield each item individually
        for item in items:
            yield item
        
        # Extract the next cursor
        next_cursor = extract_next_cursor_function(page)
        
        # Break if no more pages
        if next_cursor is None or next_cursor <= cursor or len(items) == 0:
            break
            
        cursor = next_cursor