#!/usr/bin/env python3

import asyncio
import sys
sys.path.append('.')

async def get_transaction_details():
    """Get transaction details for debugging"""
    # Import here to avoid dependency issues in the container
    import aiohttp
    import json
    
    tx_hash = "0x504aa1653fc8a7f4fc0ccd1149e63025361334af145175258aac2a7d8794992a"
    
    async with aiohttp.ClientSession() as session:
        payload = {
            "jsonrpc": "2.0",
            "method": "rooch_getTransactionsByHash",
            "params": [[tx_hash]],
            "id": 1
        }
        
        async with session.post("http://localhost:6767", json=payload) as response:
            result = await response.json()
            print(json.dumps(result, indent=2))

if __name__ == "__main__":
    asyncio.run(get_transaction_details())
