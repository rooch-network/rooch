# RoochClient

## 例子

`index.js` 的演示代码：

```js
import pkg from '@roochnetwork/rooch-sdk';
const { RoochClient, getRoochNodeUrl, Transaction } = pkg;

function main() {
    const NETWORK = 'testnet'; // 尝试修改为 'mainnet' 可连接到主网
    // 创建客户端实例
    const client = new RoochClient({
        url: getRoochNodeUrl(NETWORK)  // 连接 Rooch 网络
    });
    console.log(`Connected to Rooch ${NETWORK}`);
}

main();
```

`package.json` 的配置：

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

## 运行

```bash
npm start

> roochclient@1.0.0 start
> node index.js

Connected to Rooch testnet
```

修改 `NETWORK` 为 `mainnet` 后：

```bash
npm start

> roochclient@1.0.0 start
> node index.js

Connected to Rooch mainnet
```