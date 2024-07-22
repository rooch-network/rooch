# Bitseed SDK

## Project Introduction

The Bitseed SDK is a toolkit designed to extend the capabilities of the Ordinals protocol, facilitating the use of Inscriptions within blockchain-based games and Autonomous Worlds. Inspired by various inscription protocols such as BRC20, BRC420, BRC1024, and others, it pays homage to these pioneers. With Bitseed SDK, developers can use the hash values of Bitcoin blocks and transactions as seeds to generate unique game assets, items, characters, and worlds through a Generator.

## How to Use

### Installation

To use the Bitseed SDK, ensure that you have Node.js and npm installed on your system. You can then install the SDK with the following command:

```bash
npm install bitseed-sdk
```

Or, if you prefer to use `yarn`:

```bash
yarn add bitseed-sdk
```

### Usage Example

For usage examples, please refer to the e2e test stories located at:

- `tests/e2e/generator.story.tsx`
- `tests/e2e/deploy.story.tsx`
- `tests/e2e/mint.story.tsx`

These stories provide practical examples of how to use the SDK for deploying generators, minting assets, and more.

### E2E Testing

The Bitseed SDK includes a suite of e2e tests to ensure functionality works as expected. To run these tests, use the following command:

```bash
npm run test:e2e:debug
```

This command will execute end-to-end tests using Playwright. It's recommended to run these tests to verify that changes to the SDK do not break existing functionality.

## Contributing to Development

### Setting Up the Environment

To contribute to the Bitseed SDK, clone the repository and install dependencies:

```bash
git clone https://github.com/bitseed/bitseed.git
cd sdk/bitseed-sdk
npm install
```

### Building the Project

After making any changes, you can build the project with:

```bash
npm run build
```

### Running Unit Tests

To run unit tests, use the following command:

```bash
npm run test:unit
```

### Debugging Integration Tests

If you need to debug integration tests, you can use the following command, which enables the Playwright debug UI:

```bash
npm run test:e2e:debug
```

### Submitting Code

Before submitting your code, ensure that it passes all unit and integration tests. You can then submit a Pull Request to the main repository.

## License

The Bitseed SDK is released under the Apache License 2.0. For more details, please check the `LICENSE` file in the root directory of the project.
