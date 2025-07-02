# Rooch Python SDK User Guide

## Introduction

The Rooch Python SDK provides a comprehensive set of tools for interacting with the Rooch blockchain from Python applications. This user guide will help you understand how to use the SDK effectively.

## Installation

### Requirements

- Python 3.8 or higher
- pip (Python package manager)

### Installation Steps

```bash
# Install from PyPI
pip install rooch-sdk

# For development (install from source)
pip install -e ".[dev]"
```

## Connecting to Rooch Network

The SDK supports connecting to different Rooch network environments:

```python
import asyncio
from rooch.client.client import RoochClient
from rooch.transport import RoochEnvironment

async def main():
    # Connect to local development node
    async with RoochClient(RoochEnvironment.LOCAL) as client:
        # Use client...
        pass
        
    # Connect to testnet
    async with RoochClient(RoochEnvironment.TESTNET) as client:
        # Use client...
        pass
        
    # Connect to mainnet
    async with RoochClient(RoochEnvironment.MAINNET) as client:
        # Use client...
        pass
        
    # Connect to custom endpoint
    custom_url = "http://my-custom-rooch-node:9527"
    async with RoochClient(custom_url) as client:
        # Use client...
        pass

if __name__ == "__main__":
    asyncio.run(main())
```

## Key Management and Address Generation

```python
from rooch.crypto.keypair import KeyPair
from rooch.crypto.signer import Signer

# Generate a new random key pair
keypair = KeyPair.generate()

# Create from existing private key
private_key_hex = "0x1234..."  # Replace with your private key
keypair = KeyPair.from_private_key_hex(private_key_hex)

# Create a signer
signer = Signer(keypair)

# Get address
address = signer.get_address()
print(f"Address: {address}")

# Sign a message
message = b"Hello, Rooch!"
signature = signer.sign(message)
print(f"Signature: {signature.hex()}")

# Verify signature
is_valid = keypair.verify(message, signature)
print(f"Is signature valid? {is_valid}")
```

## Account Operations

```python
async def account_operations(client, address):
    # Get account info
    account_info = await client.account.get_account(address)
    print(f"Account info: {account_info}")
    
    # Get sequence number
    sequence_number = await client.account.get_account_sequence_number(address)
    print(f"Sequence number: {sequence_number}")
    
    # Get account balances
    balances = await client.account.get_balances(address)
    print(f"Balances: {balances}")
    
    # Get resources owned by the account
    resources = await client.account.get_resources(address)
    print(f"Resources: {resources}")
```

## Executing Move Functions

### Execute a Move Call (Transaction)

```python
async def execute_move_function(client, signer):
    # Transfer some gas coin to another address
    recipient_address = "0x123..."  # Replace with recipient address
    amount = 100  # Amount to transfer
    
    result = await client.execute_move_call(
        signer=signer,
        function_id="0x1::coin::transfer",
        type_args=["0x3::gas_coin::RGas"],
        args=[[recipient_address, str(amount)]],
        max_gas_amount=10_000_000
    )
    
    print(f"Transaction result: {result}")
    print(f"Transaction hash: {result['execution_info']['tx_hash']}")
```

### Execute a View Function (Read-Only)

```python
async def execute_view_function(client):
    # Get gas coin balance
    address = "0x123..."  # Replace with address
    
    result = await client.transaction.execute_view_function(
        function_id="0x1::coin::balance",
        type_args=["0x3::gas_coin::RGas"],
        args=[[address]]
    )
    
    print(f"View function result: {result}")
    # Extract return value (balance)
    balance = result["return_values"][0]["value"]
    print(f"Balance: {balance}")
```

## Publishing Move Modules

```python
async def publish_module(client, signer, module_path):
    # Read module bytecode
    with open(module_path, "rb") as f:
        module_bytes = f.read()
    
    # Publish the module
    result = await client.publish_module(
        signer=signer,
        module_bytes=module_bytes,
        max_gas_amount=10_000_000
    )
    
    print(f"Module published: {result}")
    print(f"Transaction hash: {result['execution_info']['tx_hash']}")
```

## WebSocket Subscriptions

```python
from rooch.client.ws_transport import RoochWebSocketTransport
from rooch.client.types.json_rpc import JsonRpcRequest

async def subscribe_to_events(client):
    # Create WebSocket transport
    ws_url = client._transport.url.replace('http', 'ws')
    ws_transport = RoochWebSocketTransport(ws_url)
    
    # Define callback
    def on_event(event):
        print(f"Received event: {event}")
    
    # Register callback
    ws_transport.on_message(on_event)
    
    # Subscribe to transaction events
    tx_request = JsonRpcRequest(
        method="rooch_subscribeTransaction",
        params=[]
    )
    tx_subscription = await ws_transport.subscribe(tx_request)
    
    # Subscribe to state changes for an address
    address = "0x123..."  # Replace with address
    state_request = JsonRpcRequest(
        method="rooch_subscribeStateChange",
        params=[address]
    )
    state_subscription = await ws_transport.subscribe(state_request)
    
    try:
        # Keep the subscription alive
        await asyncio.sleep(60)  # Listen for 60 seconds
    finally:
        # Clean up
        tx_subscription.unsubscribe()
        state_subscription.unsubscribe()
        ws_transport.destroy()
```

## Session Management

Sessions allow dApps to interact with Rooch blockchain on behalf of users with limited permissions.

```python
from rooch.session.session import SessionClient, SessionArgs

async def create_and_use_session(client, signer):
    # Create a session client
    session_client = SessionClient(client)
    
    # Create session arguments
    session_args = SessionArgs(
        app_name="My Rooch App",
        app_url="https://my-app.com",
        scopes=["0x3::empty::empty_with_signer"]  # Functions the session can call
    )
    
    # Create the session
    session = await session_client.create_session(
        session_args=session_args,
        signer=signer
    )
    
    print(f"Session created: {session}")
    
    # Use the session to execute a function
    session_signer = session_client.create_session_signer(session)
    
    result = await client.execute_move_call(
        signer=session_signer,
        function_id="0x3::empty::empty_with_signer",
        type_args=[],
        args=[],
        max_gas_amount=10_000_000
    )
    
    print(f"Session function call result: {result}")
    
    # List all sessions
    sessions = await session_client.list_sessions(signer.get_address())
    print(f"Active sessions: {sessions}")
    
    # Revoke the session
    await session_client.revoke_session(
        signer=signer,
        session_id=session["session_id"]
    )
    print("Session revoked")
```

## Transaction Building (Advanced)

For more control over transaction creation:

```python
async def build_transaction(client, signer):
    # Get a transaction builder
    tx_builder = await client.get_transaction_builder(
        sender_address=signer.get_address(),
        max_gas_amount=10_000_000,
        gas_unit_price=1,
        expiration_delta_secs=600  # 10 minutes
    )
    
    # Build function payload
    recipient_address = "0x456..."  # Replace with recipient address
    amount = 100  # Amount to transfer
    
    payload = tx_builder.build_function_payload(
        function_id="0x1::coin::transfer",
        ty_args=["0x3::gas_coin::RGas"],
        args=[[recipient_address, str(amount)]]
    )
    
    # Build transaction data
    tx_data = tx_builder.build_move_action_tx(payload)
    
    # Sign transaction
    signed_tx = tx_builder.sign(tx_data, signer)
    
    # Submit transaction
    result = await client.transaction.submit_transaction(signed_tx)
    print(f"Transaction result: {result}")
```

## Error Handling

The SDK provides a comprehensive error hierarchy:

```python
from rooch.client.error import (
    RoochError, 
    RoochTransportError, 
    RoochTimeoutError,
    RoochRpcError,
    RoochTxExecutionError
)

async def error_handling_example(client, signer):
    try:
        # Execute a function that might fail
        await client.execute_move_call(
            signer=signer,
            function_id="0x1::non_existent::function",
            type_args=[],
            args=[],
            max_gas_amount=10_000_000
        )
    except RoochTxExecutionError as e:
        print(f"Transaction execution error: {e}")
        print(f"TX hash: {e.tx_hash}")
        print(f"VM status: {e.vm_status}")
    except RoochRpcError as e:
        print(f"RPC error: {e}")
        print(f"Error code: {e.code}")
    except RoochTimeoutError as e:
        print(f"Timeout error: {e}")
        print(f"Timeout: {e.timeout_ms}ms")
        print(f"Method: {e.request_method}")
    except RoochTransportError as e:
        print(f"Transport error: {e}")
        print(f"URL: {e.url}")
    except RoochError as e:
        print(f"General error: {e}")
```

## Working with BCS (Binary Canonical Serialization)

```python
from rooch.bcs.serializer import BcsSerializer

# Create a serializer
serializer = BcsSerializer()

# Serialize basic types
uint8_bytes = serializer.serialize_u8(255)
uint64_bytes = serializer.serialize_u64(1000000)
string_bytes = serializer.serialize_str("Hello, Rooch!")
bool_bytes = serializer.serialize_bool(True)

# Serialize vector/list
vector_bytes = serializer.serialize_vector([1, 2, 3, 4, 5], serializer.serialize_u32)

# Custom struct serialization
class MyStruct:
    def __init__(self, a: int, b: str, c: bool):
        self.a = a
        self.b = b
        self.c = c
        
    def serialize(self, serializer):
        serializer.serialize_u32(self.a)
        serializer.serialize_str(self.b)
        serializer.serialize_bool(self.c)
        
my_struct = MyStruct(42, "answer", True)
struct_bytes = serializer.serialize(my_struct)
```

## Best Practices

1. **Always use context managers** (`async with`) to properly close connections:
   ```python
   async with RoochClient(RoochEnvironment.TESTNET) as client:
       # Your code here
   ```

2. **Handle errors gracefully** by catching specific exception types:
   ```python
   try:
       # Your code here
   except RoochTxExecutionError as e:
       # Handle transaction execution errors
   except RoochError as e:
       # Handle other SDK errors
   ```

3. **Use pagination** for large result sets:
   ```python
   cursor = 0
   limit = 25
   while True:
       result = await client.get_states(cursor=cursor, limit=limit)
       # Process result["data"]
       if not result["data"] or cursor >= result["cursor"]:
           break
       cursor += limit
   ```

4. **Reuse signer instances** rather than creating new ones for each transaction.

5. **Close WebSocket connections** when finished:
   ```python
   try:
       # Your WebSocket code
   finally:
       ws_transport.destroy()
   ```

## Troubleshooting

### Common Issues and Solutions

#### "Failed to connect to node"
- Check if the node is running
- Verify the URL is correct
- Ensure network connectivity

#### "Insufficient gas"
- Increase the `max_gas_amount` parameter
- Ensure the account has enough gas tokens

#### "Invalid account authentication key"
- Ensure you're using the correct private key for the account

#### "Transaction timed out"
- The network might be congested, try again
- Consider increasing the client timeout

#### "Unable to parse response"
- The SDK might be incompatible with the node version
- Update to the latest SDK version

## Advanced Topics

### Custom Authentication Validators

```python
# TODO: Add example of custom authentication validators
```

### Batch Transaction Processing

```python
# TODO: Add example of batch transaction processing
```

### Gas Estimation

```python
# TODO: Add example of gas estimation
```

## API Reference

For a complete API reference, please see the [API Documentation](https://rooch.network/docs/python-sdk-api).

## Contributing

Contributions to the Rooch Python SDK are welcome! Please see our [Contributing Guide](https://github.com/rooch-network/rooch/blob/main/CONTRIBUTING.md) for more information.

## License

The Rooch Python SDK is licensed under the Apache License, Version 2.0. See [LICENSE](https://github.com/rooch-network/rooch/blob/main/LICENSE) for details.
