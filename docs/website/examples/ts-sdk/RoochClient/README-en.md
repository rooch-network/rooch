# RoochClient

## Example

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