# Onchain AI Chat

A decentralized chat application built on Rooch blockchain with AI integration through Verity Oracle.

## Features

- Create and join chat rooms
- Real-time messaging with other users
- AI-powered chat rooms with GPT-4 integration
- On-chain message storage and persistence
- Pagination support for message history
- Real-time message updates

## Architecture

### Smart Contracts

- `room.move`: Main chat room functionality including message handling
- `ai_service.move`: AI integration service using Verity Oracle
- `ai_callback.move`: Handles AI response callbacks

### Frontend

- React-based web interface
- Real-time updates using Rooch SDK
- Material design UI components
- Message pagination and infinite scroll

## Prerequisites

- [Rooch](https://rooch.network) development environment
- Node.js v16+ and npm/yarn
- Move compiler

## Getting Started

1. Clone the repository:
```bash
git clone https://github.com/rooch-network/rooch.git
cd rooch/examples/onchain_ai_chat
```

2. Deploy the smart contracts:

```bash
rooch move publish --named-addresses onchain_ai_chat=default
```

3. Start the frontend:

```bash
cd web
pnpm install
pnpm dev
```
More details can be found in the [web README](web/README.md).
