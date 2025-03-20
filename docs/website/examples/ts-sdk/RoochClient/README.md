# RoochClient

## Examples

Demonstration code for `index.js`:

```js
import pkg from '@roochnetwork/rooch-sdk';
const { RoochClient, getRoochNodeUrl, Transaction } = pkg;

function main() {
    const NETWORK = 'testnet'; // Try changing to 'mainnet' to connect to the main network
    // Create a client instance
    const client = new RoochClient({
        url: getRoochNodeUrl(NETWORK)  // Connect to the Rooch network
    });
    console.log(`Connected to Rooch ${NETWORK}`);
}

main();
```

Configuration for `package.json`:

```json
{
  "name": "roochclient",
  "version": "1.0.0",
  "main": "index.js",
  "type": "module",
  "scripts": {
    "start": "node index.js"
  },
  "dependencies": {
    "@roochnetwork/rooch-sdk": "latest"
  },
  "keywords": [],
  "author": "",
  "license": "ISC",
  "description": ""
}
```

## Run

```bash
npm start

> roochclient@1.0.0 start
> node index.js

Connected to Rooch testnet
```

After changing `NETWORK` to `mainnet`:

```bash
npm start

> roochclient@1.0.0 start
> node index.js

Connected to Rooch mainnet
```