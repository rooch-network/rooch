# 存款流程

## 概述

存款在 Rooch 语境中指代一切由 L1 触发的 L2 交易。（并不局限于资产转移）

Rooch 具备多链资产结算的能力，对于每一条 L1 来说，其存款流程一致（合约及其参数受不同智能合约语言影响略有不同，Rooch 将）。都是从 L1
的消息开始经过层层封装传递到 L2 对应的合约之中进行消费：

<img alt="Rooch Deposit Flow" height="300" src="/docs/deposit_flow.jpeg" width="600"/>

我们需要保证来自 L1 的消息能够 `rooch_node` 正确的解析并中继给相应的合约。

## L1

1. 用户通过 `send_msg` 函数向 `l1_messenger` 发起存款请求，所需参数包括:
   1. `target`: L2 合约地址
   2. `msg`: L2 tx 的 calldata 
   3. `gas`: TODO gas 机制设计

2. `l1_messenger` 将调用 `rooch_gateway` 的 `send_to_rooch` 函数，
   `send_to_rooch` 为 L1 向 L2 传递消息的底层函数，其参数包括：
   1. `to`: l2_messenger 合约地址
   2. `gas`: TODO gas 机制设计
   3. `data`: TODO relay message data 字段设计(如所需要传递的`target` `msg` 等)

3. `send_to_rooch` 在完成参数封装和检查后，发出 `TxDepositedEvent` 事件

## L2

1. `rooch_node` 监听 `TxDepositedEvent` 事件，解析其中的参数，并封装为 `deposit_tx`
2. 将 `deposit_tx` 传递给 `l2_messenger` 的 `relay_message` 函数 
3. `relay_message` 在完成检查后，调用 `target` 合约



