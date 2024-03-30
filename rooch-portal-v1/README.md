# Rooch Portal

## Overview

Rooch Portal is a comprehensive dashboard designed to simplify the management of Bitcoin wallets, such as Unisat, and to provide users with an intuitive interface to manage their Bitcoin and Rooch accounts. This portal offers a centralized platform for tracking and organizing all assets, inscriptions, and other related financial activities.

### Features

- Wallet Connection: Easily connect and manage your Bitcoin wallets, such as Unisat.
- Asset Management: View and manage your Bitcoin and Rooch account assets in one place.
- Inscription Tracking: Keep track of all your inscriptions and related activities.
- User-Friendly Interface: Navigate through your financial information with an intuitive and user-friendly dashboard.
- Secure and Reliable: Prioritizes the security and privacy of your financial data.

## Getting Started

### Prerequisites

- Node.js
- npm or yarn
- A Bitcoin wallet (e.g., Unisat)

### Installation

1. Clone the repository:

```bash
git clone https://github.com/TwilightLogic/rooch-portal-v1.git
```

2. Navigate to the project directory:

## Run Locally

To get started you need to install [pnpm](https://pnpm.io/), then run the following command:

```bash
# Install all dependencies
pnpm install
# Run the build for the TypeScript SDK
pnpm rooch-sdk gen
pnpm rooch-sdk build
# Run the build for the TypeScript SDK Kit
pnpm rooch-sdk-kit build
# Run the build for the
pnpm rooch-portal-v1 dev

```

> All `pnpm` commands are intended to be run in the root of the Rooch repo. You can also run them within the `sdk/typescript` directory, and remove change `pnpm sdk` to just `pnpm` when running commands.

## Usage

After launching Rooch Portal, follow these steps:

1. Connect your Bitcoin wallet (e.g., Unisat) using the 'Connect Wallet' option.
2. Once connected, navigate through the dashboard to view your assets and inscriptions.
3. Use the provided tools and features to manage and organize your financial data effectively.

## Test
