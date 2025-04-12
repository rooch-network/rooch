#!/usr/bin/env python3
# Copyright (c) Rooch Network
# SPDX-License-Identifier: Apache-2.0

"""
Example of using WebSocket subscriptions with the Rooch Python SDK.
This demonstrates how to subscribe to transaction and event updates in real-time.
"""

import asyncio
import json
import signal
from typing import Dict, Any, Optional

from rooch.client.client import RoochClient
from rooch.client.ws_transport import RoochWebSocketTransport
from rooch.transport import RoochEnvironment


async def transaction_handler(tx_data: Dict[str, Any]) -> None:
    """Handler for transaction updates
    
    Args:
        tx_data: Transaction data
    """
    print("\n=== Received Transaction Update ===")
    tx_hash = tx_data.get("transaction_hash", "unknown")
    sender = tx_data.get("sender", "unknown")
    status = tx_data.get("status", "unknown")
    
    print(f"Transaction Hash: {tx_hash}")
    print(f"Sender: {sender}")
    print(f"Status: {status}")
    
    # For brevity, we don't print the full transaction data
    # Uncomment to see all transaction details
    # print(f"Full data: {json.dumps(tx_data, indent=2)}")


async def event_handler(event_data: Dict[str, Any]) -> None:
    """Handler for event updates
    
    Args:
        event_data: Event data
    """
    print("\n=== Received Event Update ===")
    event_type = event_data.get("type_tag", "unknown")
    sender = event_data.get("sender", "unknown")
    sequence_number = event_data.get("sequence_number", "unknown")
    
    print(f"Event Type: {event_type}")
    print(f"Sender: {sender}")
    print(f"Sequence Number: {sequence_number}")
    
    # For brevity, we don't print the full event data
    # Uncomment to see all event details
    # print(f"Full data: {json.dumps(event_data, indent=2)}")
    
    # If you want to parse the event data based on type:
    # if "coin::TransferEvent" in event_type:
    #     print("This is a coin transfer event")
    #     # Parse specific coin transfer fields
    # elif "some_other_event" in event_type:
    #     # Handle other event types


async def subscribe_to_transactions(client: RoochClient, filter_sender: Optional[str] = None) -> str:
    """Subscribe to transaction updates
    
    Args:
        client: Rooch client
        filter_sender: Optional sender address to filter by
        
    Returns:
        Subscription ID
    """
    print("\n=== Subscribing to transactions ===")
    
    filter_params = {"sender": filter_sender} if filter_sender else {}
    
    try:
        subscription = await client.subscribe(
            type="transaction",
            filter=filter_params,
            on_message=transaction_handler,
            on_error=lambda error: print(f"Subscription error: {error}")
        )
        
        print(f"Subscribed to transactions with ID: {subscription.id}")
        return subscription.id
    except Exception as e:
        print(f"Error subscribing to transactions: {e}")
        return ""


async def subscribe_to_events(client: RoochClient, filter_sender: Optional[str] = None) -> str:
    """Subscribe to event updates
    
    Args:
        client: Rooch client
        filter_sender: Optional sender address to filter by
        
    Returns:
        Subscription ID
    """
    print("\n=== Subscribing to events ===")
    
    filter_params = {"sender": filter_sender} if filter_sender else {}
    
    try:
        subscription = await client.subscribe(
            type="event",
            filter=filter_params, 
            on_message=event_handler,
            on_error=lambda error: print(f"Subscription error: {error}")
        )
        
        print(f"Subscribed to events with ID: {subscription.id}")
        return subscription.id
    except Exception as e:
        print(f"Error subscribing to events: {e}")
        return ""


async def main() -> None:
    """Main function"""
    # Handle graceful shutdown
    shutdown_event = asyncio.Event()
    
    def signal_handler():
        print("\n=== Shutting down ===")
        shutdown_event.set()
        
    # Register signal handlers
    loop = asyncio.get_running_loop()
    for sig in (signal.SIGINT, signal.SIGTERM):
        loop.add_signal_handler(sig, signal_handler)
    
    try:
        # Initialize WebSocket transport
        ws_transport = RoochWebSocketTransport(
            url=RoochEnvironment.LOCAL,
            reconnect_delay=1000,  # 1 second between reconnection attempts
            max_reconnect_attempts=5
        )
        
        # Initialize client with WebSocket transport
        client = RoochClient(
            transport=ws_transport,
            subscription_transport=ws_transport
        )
        
        print("=== Connecting to Rooch node via WebSocket ===")
        
        # Subscribe to transactions and events
        tx_subscription_id = await subscribe_to_transactions(client)
        event_subscription_id = await subscribe_to_events(client)
        
        print("\n=== Waiting for updates ===")
        print("Press Ctrl+C to exit")
        
        # Keep the script running until shutdown is requested
        await shutdown_event.wait()
        
        # Unsubscribe and cleanup
        if tx_subscription_id:
            await client.unsubscribe(tx_subscription_id)
        
        if event_subscription_id:
            await client.unsubscribe(event_subscription_id)
        
        # Destroy the client to clean up resources
        await client.close()
        
    except Exception as e:
        print(f"Error: {e}")


if __name__ == "__main__":
    asyncio.run(main())