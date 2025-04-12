# Rooch Python SDK

The official Python SDK for Rooch Network.

## Features

- **Complete API Coverage**: Access the full suite of Rooch functionality through a simple Python interface
- **Asynchronous Design**: Built with modern async/await pattern for high-performance applications
- **Type-Safe**: Comprehensive typing for better IDE support and fewer runtime errors
- **BCS Serialization**: Built-in support for Move's Binary Canonical Serialization format
- **Transaction Building**: Utilities to easily create, sign, and submit transactions
- **Address Management**: Support for Rooch addresses and Bitcoin addresses
- **Key Management**: Secure key generation and management

## Installation

```bash
pip install rooch
```

For development:

```bash
pip install -e ".[dev]"
```

## Quick Start

```python
import asyncio
from rooch.client.client import RoochClient
from rooch.transport import RoochEnvironment
from rooch.crypto.keypair import KeyPair
from rooch.crypto.signer import Signer

async def main():
    # Connect to Rooch testnet
    async with RoochClient(RoochEnvironment.TESTNET) as client:
        # Generate a new key pair
        keypair = KeyPair.generate()
        signer = Signer(keypair)
        
        # Get the address
        address = signer.get_address()
        print(f"Generated address: {address}")
        
        # Get account info
        account_info = await client.account.get_account(address)
        print(f"Account info: {account_info}")
        
        # Get account balance
        balance = await client.account.get_balances(address)
        print(f"Balances: {balance}")
        
        # Execute a Move call
        result = await client.execute_move_call(
            signer=signer,
            function_id="0x1::coin::transfer",
            type_args=["0x1::coin::ROOCH"],
            args=[["0x123...456", "100"]],
            max_gas_amount=10_000_000
        )
        print(f"Transaction result: {result}")

if __name__ == "__main__":
    asyncio.run(main())
```

## Documentation

For detailed documentation, see [Rooch Documentation](https://rooch.network/docs).

## Examples

Check out the `examples` directory for more usage examples:

- Basic usage
- Account management
- Transaction handling
- Smart contract interaction
- And more...

## Development

### Requirements

- Python 3.8+
- pip

### Setting Up Development Environment

1. Clone the repository
2. Create and activate a virtual environment (recommended):
   ```bash
   python -m venv venv
   source venv/bin/activate  # On Windows: venv\Scripts\activate
   ```
3. Install development dependencies: `pip install -e ".[dev]"`
4. Run tests: `pytest`

## License

Apache License 2.0
