//import { RoochClient, getRoochNodeUrl, Transaction } from '@roochnetwork/rooch-sdk';
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