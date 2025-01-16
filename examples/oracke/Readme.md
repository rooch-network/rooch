# Rooch Oracle Example

This example demonstrates how to use and interact with Oracles in the Rooch Network.

## Documentation

For detailed documentation on setting up and using Oracles in Rooch Network, including:

- Prerequisites
- Step-by-step setup instructions
- Account creation
- Contract deployment
- Environment configuration
- URL management
- Running the orchestrator
- Making oracle requests
- Managing escrow balance

Please refer to our comprehensive guide in [docs/ROOCH.md](https://github.com/usherlabs/verity-move-oracles/blob/main/docs/ROOCH.md).

## Quick Start

1. Follow the setup instructions in [docs/ROOCH.md](https://github.com/usherlabs/verity-move-oracles/blob/main/docs/ROOCH.md)
2. Deploy the oracle contracts
3. Configure your environment
4. Run the orchestrator
5. Make oracle requests

## Supported APIs

### Twitter/X API
- User Endpoint: `https://api.x.com/2/users/` and `https://api.x.com/2/tweets/`
  ```bash
  # Example: Get user followers count
  rooch move run --function <oracle_address>::example_caller::request_data \
    --sender-account default \
    --args 'string:https://api.x.com/2/users/by/username/elonmusk?user.fields=public_metrics' \
    --args 'string:GET' \
    --args 'string:{}' \
    --args 'string:{}' \
    --args 'string:.data.public_metrics.followers_count' \
    --args 'address:<orchestrator_address>' \
    --args 'u256:50000000'
  ```

### OpenAI API
- Chat Completions: `https://api.openai.com/v1/chat/completions`
  ```bash
  # Example: Simple GPT request
  rooch move run --function <oracle_address>::example_caller::request_data \
    --sender-account default \
    --args 'string:https://api.openai.com/v1/chat/completions' \
    --args 'string:POST' \
    --args 'string:{}' \
    --args 'string:{
      "model": "gpt-4",
      "messages": [{"role": "user", "content": "Say this is a test!"}],
      "temperature": 0.7
    }' \
    --args 'string:.choices[].message.content' \
    --args 'address:<orchestrator_address>' \
    --args 'u256:50000000'
  ```

Note: Replace `<oracle_address>` and `<orchestrator_address>` with your actual deployed addresses.

## Support

For additional help or questions, please refer to the main documentation or open an issue in the [repository](https://github.com/usherlabs/verity-move-oracles.git).