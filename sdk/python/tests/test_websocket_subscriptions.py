import pytest
import pytest_asyncio
import asyncio
from typing import Dict, Any, List, Set

from rooch.client.client import RoochClient
from rooch.crypto.keypair import KeyPair
from rooch.crypto.signer import Signer
from rooch.client.ws_transport import RoochWebSocketTransport
from rooch.client.subscription_interface import Subscription
from rooch.client.types.json_rpc import JsonRpcRequest

# Mark all tests in this module as integration tests
pytestmark = pytest.mark.integration

class TestWebSocketSubscriptions:
    """Tests for WebSocket subscriptions in the SDK"""

    @pytest.mark.asyncio
    async def test_transaction_subscription(self, rooch_client: RoochClient, test_signer: Signer):
        """Test subscribing to transaction events"""
        # Create a WebSocket transport
        ws_url = rooch_client._transport.url.replace('http', 'ws')
        ws_transport = RoochWebSocketTransport(ws_url)
        
        # Create event collection for verification
        received_events = []
        event_received = asyncio.Event()
        
        # Define callback
        def on_transaction(event: Dict[str, Any]):
            print(f"Received transaction event: {event}")
            received_events.append(event)
            event_received.set()
        
        # Register message callback
        ws_transport.on_message(on_transaction)
        
        # Subscribe to transaction events
        request = JsonRpcRequest(
            method="rooch_subscribeTransaction",
            params=[]
        )
        subscription = await ws_transport.subscribe(request)
        
        try:
            # Generate a transaction to trigger an event
            # Transfer a small amount to self to generate activity
            await rooch_client.execute_move_call(
                signer=test_signer,
                function_id="0x3::transfer::transfer_coin", 
                type_args=["0x3::gas_coin::GasCoin"],
                args=[[test_signer.get_address(), 1]],  # Transfer 1 token to self
                max_gas_amount=10_000_000
            )
            
            # Wait for the event to be received or timeout
            try:
                await asyncio.wait_for(event_received.wait(), timeout=10.0)
                assert len(received_events) > 0
                
                # Validate event structure
                event = received_events[0]
                assert "execution_info" in event or "tx_hash" in event
                
            except asyncio.TimeoutError:
                pytest.fail("Timed out waiting for transaction event")
                
        finally:
            # Unsubscribe and clean up
            subscription.unsubscribe()
            ws_transport.destroy()

    @pytest.mark.asyncio
    async def test_state_subscription(self, rooch_client: RoochClient, test_signer: Signer):
        """Test subscribing to state changes"""
        # Create a WebSocket transport
        ws_url = rooch_client._transport.url.replace('http', 'ws')
        ws_transport = RoochWebSocketTransport(ws_url)
        
        # Create event collection for verification
        received_events = []
        event_received = asyncio.Event()
        
        # Define callback
        def on_state_change(event: Dict[str, Any]):
            print(f"Received state change event: {event}")
            received_events.append(event)
            event_received.set()
        
        # Register message callback
        ws_transport.on_message(on_state_change)
        
        # Subscribe to state changes for the test account
        address = test_signer.get_address()
        request = JsonRpcRequest(
            method="rooch_subscribeStateChange",
            params=[address]
        )
        subscription = await ws_transport.subscribe(request)
        
        try:
            # Generate a transaction that modifies state
            # Transfer a small amount to self to generate state change
            await rooch_client.execute_move_call(
                signer=test_signer,
                function_id="0x3::transfer::transfer_coin", 
                type_args=["0x3::gas_coin::GasCoin"],
                args=[[test_signer.get_address(), 1]],  # Transfer 1 token to self
                max_gas_amount=10_000_000
            )
            
            # Wait for the event to be received or timeout
            try:
                await asyncio.wait_for(event_received.wait(), timeout=10.0)
                assert len(received_events) > 0
                
                # Validate event structure
                event = received_events[0]
                assert "states" in event or "state_key" in event
                
            except asyncio.TimeoutError:
                pytest.fail("Timed out waiting for state change event")
                
        finally:
            # Unsubscribe and clean up
            subscription.unsubscribe()
            ws_transport.destroy()

    @pytest.mark.asyncio
    async def test_event_subscription(self, rooch_client: RoochClient, test_signer: Signer):
        """Test subscribing to Move events"""
        # Create a WebSocket transport
        ws_url = rooch_client._transport.url.replace('http', 'ws')
        ws_transport = RoochWebSocketTransport(ws_url)
        
        # Create event collection for verification
        received_events = []
        event_received = asyncio.Event()
        
        # Define callback
        def on_move_event(event: Dict[str, Any]):
            print(f"Received Move event: {event}")
            received_events.append(event)
            event_received.set()
        
        # Register message callback
        ws_transport.on_message(on_move_event)
        
        # Subscribe to all events (or filter by type if supported)
        request = JsonRpcRequest(
            method="rooch_subscribeEvent",
            params=[]  # Empty params means all events
        )
        subscription = await ws_transport.subscribe(request)
        
        try:
            # Generate a transaction that emits events
            # Transfer a small amount which should emit events
            await rooch_client.execute_move_call(
                signer=test_signer,
                function_id="0x3::transfer::transfer_coin", 
                type_args=["0x3::gas_coin::GasCoin"],
                args=[[test_signer.get_address(), 1]],  # Transfer 1 token to self
                max_gas_amount=10_000_000
            )
            
            # Wait for the event to be received or timeout
            try:
                await asyncio.wait_for(event_received.wait(), timeout=10.0)
                assert len(received_events) > 0
                
                # Validate event structure
                event = received_events[0]
                assert "event_data" in event or "type_tag" in event
                
            except asyncio.TimeoutError:
                pytest.fail("Timed out waiting for Move event")
                
        finally:
            # Unsubscribe and clean up
            subscription.unsubscribe()
            ws_transport.destroy()

    @pytest.mark.asyncio
    async def test_multiple_subscriptions(self, rooch_client: RoochClient, test_signer: Signer):
        """Test managing multiple subscriptions simultaneously"""
        # Create a WebSocket transport
        ws_url = rooch_client._transport.url.replace('http', 'ws')
        ws_transport = RoochWebSocketTransport(ws_url)
        
        # Create event collection for verification
        received_tx_events = []
        received_state_events = []
        
        tx_event_received = asyncio.Event()
        state_event_received = asyncio.Event()
        
        # Define a single callback that handles both event types
        def on_event(event: Dict[str, Any]):
            print(f"Received event: {event}")
            
            # Check event type and route accordingly
            if "method" in event:
                method = event.get("method", "")
                if "transaction" in method.lower():
                    received_tx_events.append(event)
                    tx_event_received.set()
                elif "state" in method.lower():
                    received_state_events.append(event)
                    state_event_received.set()
            
        # Register message callback
        ws_transport.on_message(on_event)
        
        # Subscribe to both transaction and state events
        tx_request = JsonRpcRequest(
            method="rooch_subscribeTransaction",
            params=[]
        )
        tx_subscription = await ws_transport.subscribe(tx_request)
        
        address = test_signer.get_address()
        state_request = JsonRpcRequest(
            method="rooch_subscribeStateChange",
            params=[address]
        )
        state_subscription = await ws_transport.subscribe(state_request)
        
        try:
            # Generate a transaction that should trigger both subscriptions
            await rooch_client.execute_move_call(
                signer=test_signer,
                function_id="0x3::transfer::transfer_coin", 
                type_args=["0x3::gas_coin::GasCoin"],
                args=[[test_signer.get_address(), 1]],  # Transfer 1 token to self
                max_gas_amount=10_000_000
            )
            
            # Wait for both events or timeout
            try:
                await asyncio.wait_for(
                    asyncio.gather(
                        tx_event_received.wait(),
                        state_event_received.wait()
                    ),
                    timeout=15.0
                )
                
                assert len(received_tx_events) > 0
                assert len(received_state_events) > 0
                
            except asyncio.TimeoutError:
                # Check which events we received
                if not tx_event_received.is_set():
                    pytest.fail("Timed out waiting for transaction event")
                if not state_event_received.is_set():
                    pytest.fail("Timed out waiting for state change event")
                
        finally:
            # Unsubscribe and clean up
            tx_subscription.unsubscribe()
            state_subscription.unsubscribe()
            ws_transport.destroy()
