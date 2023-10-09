# Counter Web Example

## Introduction

This is a simple web application example that demonstrates how to interact with the Ethereum blockchain using MetaMask SDK and Rooch SDK. The application includes a counter that can be incremented by interacting with a smart contract on the Ethereum blockchain.

## Getting Started

### Prerequisites

- Node.js and npm installed on your machine.
- MetaMask browser extension installed and set up.

### Installation

1. Clone the repository:
   ```
   git clone <repository-url>
   ```
2. Navigate to the project directory:
   ```
   cd sdk/typescript/examples/counter
   ```
3. Install the dependencies:
   ```
   pnpm install
   ```

### Usage

1. Start the application:
   ```
   pnpm dev
   ```
2. Open your web browser and navigate to `http://localhost:3000`.
3. Connect your MetaMask wallet.
4. You can now interact with the counter on the web page.

If you have deployed your own Counter contract, you can replace the default counter address by adding a `counter_address` query parameter to the URL. For example, `http://localhost:3000?counter_address=your_counter_address`.

## Code Overview

The main application logic is located in the `App.tsx` file. It uses the MetaMask SDK to interact with the Ethereum blockchain. The `counterAddress` is retrieved from the URL parameters, and if it's not provided, a default address is used.

When the "Inc" button is clicked, a transaction is sent to the Ethereum blockchain to increment the counter in the smart contract. The counter state in the application is then updated.

## Contributing

Contributions are welcome! Please feel free to submit a pull request.

## License

Current Rooch code is released under [Apache 2.0](/LICENSE).
