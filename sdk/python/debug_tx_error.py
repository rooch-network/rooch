#!/usr/bin/env python3

import asyncio
import sys
sys.path.append('.')

async def get_transaction_error_details():
    """Get detailed error information for a failed transaction"""
    import aiohttp
    import json
    
    tx_hash = "0xc654460a54dd137391ceaf3382199c57835657ffb1a0979931b4239616a0611d"
    
    async with aiohttp.ClientSession() as session:
        # Try to get more detailed error info
        payload = {
            "jsonrpc": "2.0",
            "method": "rooch_getEvents",
            "params": [{
                "tx_hash": tx_hash
            }],
            "id": 1
        }
        
        async with session.post("http://localhost:6767", json=payload) as response:
            result = await response.json()
            print("=== Events for transaction ===")
            print(json.dumps(result, indent=2))
            
        # Also try to get state changes
        payload2 = {
            "jsonrpc": "2.0", 
            "method": "rooch_getTransactionsByHash",
            "params": [[tx_hash], {"with_full_state_change": True}],
            "id": 2
        }
        
        async with session.post("http://localhost:6767", json=payload2) as response:
            result2 = await response.json()
            print("\n=== Transaction with state changes ===")
            print(json.dumps(result2, indent=2))

if __name__ == "__main__":
    asyncio.run(get_transaction_error_details())
