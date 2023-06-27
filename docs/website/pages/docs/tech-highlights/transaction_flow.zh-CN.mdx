# 交易执行流

本文档主要目标是解释 Rooch 中交易的处理流程，让 DApp 开发者以及 Rooch 的开发者深入理解 Rooch 的设计和实现，从而更容易的参与到 DApp 以及 Rooch 的开发中。同时，本文档也试图解答一些常见问题，比如交易的执行顺序，交易的最终确定性等。

## 用户视角

从用户的调用视角来看，Rooch 的交易执行流程如下：

![rooch transaction flow user perspective](../static/design/rooch-design-transaction-flow-user-perspective.svg)

1. 用户通过 SDK，或者 CLI 将交易发送到 Rooch RPC API。同时，Rooch 也支持 Ethereum RPC API，所以用户也可以通过 MetaMask 等支持 Ethereum RPC 的客户端发送交易给 Rooch。未来，Rooch 会支持更多 L1 的 RPC API。
2. Rooch 的各种 RPC API 收到交易后，统一发送给 RPC Service 进行处理。
3. RPC Service 会调用 Executor, Sequencer, Proposer 等模块，对交易进行处理。
4. Sequencer 以及 Proposer 会定时批量和后面的 L1 交互。 

## 功能视角

从系统内部组件的功能视角来看，Rooch 的交易执行流程如下：

![rooch transaction flow functional perspective](../static/design/rooch-design-transaction-flow-functional-perspective.svg)

1. RPC Service 收到不同的 API 的交易后，会先将多链交易发送给 Executor 进行 `validate_tx`。
2. 在 Executor 的 `validate_tx` 中：
    * 先调用 `rooch_framework::address_mapping::resolve` 方法，将多链地址转换成 Rooch 地址（Move 地址）。
    * 然后调用 `rooch_framework::transaction_validator::validate` 方法对交易的 `Authenticator` 进行验证。`Authenticator` 代表交易的 `sender` 对自己身份的证明，通常是一种签名。`Authenticator` 中包含一个 `scheme` 字段来标志 `Authenticator` 的类型。当前支持 `ED25519` 和 Ethereum 的 `ECDSA` 两种内置的 `Authenticator`，未来会支持更多的签名类型，以及允许开发者自定义 `Authenticator`。这也是 AccountAbstraction 的一部分。另外账户的 `sequence_number`(相当于 Ethereum 中的 `nonce`) 也会在 `validate` 中进行验证。
    * 通过合约对交易验证后，Executor 基于交易构造 `TxContext`，将多链交易统一转换为 `MoveOSTransaction`, 并返回。
3. RPC Service 收到 Executor 的 `validate_tx` 返回值后进行判断，如果交易未验证通过，则直接给用户返回错误，否则将交易发送给 Sequencer 进行 `sequence_tx`。因为 `validate_tx` 是只读方法，不会修改状态，所以这一步返回错误并没有副作用。
4. Sequencer 将交易添加到 Accumulator 中，获得该交易的 `tx_order`，构造 `TransactionSequenceInfo`。`TransactionSequenceInfo` 包含 Sequencer 对该交易的 `tx_order` 的签名，以及 `tx_accumulator_root`。Sequencer 会定时批量将交易提交给 DA 。
5. RPC Service 收到 Sequencer 的 `sequence_tx` 返回值后，将交易发送给 Executor 进行 `execute_tx`。
6. 在 Executor 的 `execute_tx` 中，Executor 直接调用 MoveOS 执行交易。
    * 首先，MoveOS 会执行 `rooch_framework::transaction_validator::pre_execute` 方法，对交易进行前置处理。前置处理中，当前会自动创建账户以及对多链地址和 Move 地址进行映射。未来，AccountAbstraction 中的 Gas 费用相关的需求也会在前置处理中实现，比如 Gas 交换以及代付。
    * 然后，MoveOS 会调用用户定义的方法，执行交易。
    * 最后，MoveOS 会执行 `rooch_framework::transaction_validator::post_execute` 方法，进行后置处理。后置处理中，当前会自动更新账户的 `sequence_number`，以及 Gas 费用的扣取。
    * 执行过程中，`pre_execute`,`execute`,`post_execute` 共享一个 `TxContext`, 可以通过 `TxContext` 传递数据。
    * 注意，如果执行过程中，用户定义的方法执行失败，MoveOS 会自动回滚状态，但 `pre_execute` 和 `post_execute` 依然会执行，Gas 费用只扣取用户实际执行消耗的部分。
7. RPC Service 收到 `execute_tx` 返回的 `TransactionExecutionInfo` 后，将交易发送给 Proposer 进行 `propose_tx`。Proposer 将交易打包成区块，定时将区块提交到 L1 的 StateCommitment 合约。注意，这里的区块并不包含交易的原始数据，相当于区块头，它包含 Rooch 的 `state_root` 以及交易的 `tx_accumulator_root`。
8. 最后 RPC Service 将 `TransactionSequenceInfo` 和 `TransactionExecutionInfo` 返回给用户，代表交易执行成功并确认。

**注意：**
1. 当前的这个流程中，并没有包含 Challenger 以及 fraud-proof, zk-proof 的逻辑。这部分内容会在后续的版本中更新。
2. 整个流程包含了 `Executor`,`Sequencer`,`Proposer`多个组件，但这些组件可能并不在同一个节点中，它们可能是远程 P2P 网络通信。这部分内容也会在后续的版本中更新。
3. 以上的流程基于当前版本的设计来描述，并且部分逻辑尚未完全实现，未来会持续更新。

## FAQ

### 交易的执行顺序是怎么确定的？
    
交易的执行顺序是由 Sequencer 确定的。Sequencer 会将交易添加到 Accumulator 中，并立刻获取到交易的全局顺序，Sequencer 需要对该交易的顺序进行签名，相当于给用户一个承诺，承诺自己不会修改顺序或者丢弃交易。而 Accumulator 可以提供交易的顺序证明，如果最后 Sequencer 提交到 DA 的交易顺序和之前的承诺不一致，那么用户可以证明 Sequencer 的行为是恶意的，从对 Sequencer 进行惩罚。 

### 交易的执行结果是怎么确定的？

Rooch 中没有交易池的概念，交易的执行结果是即时确定的，客户端提交交易后会立刻得到结果，不需要等待异步共识确认。因为如果交易的执行顺序是确定的，程序是确定的，那么交易的执行结果也是确定的。这里面包含一些安全假设，因为 L2 的安全假设构建在[反事实因果推理](https://en.wikipedia.org/wiki/Counterfactual_thinking)基础之上，如果作弊行为会得到惩罚，理性的选择是不作弊。

1. 交易的执行顺序是由 Sequencer 保证的，用户通过前面提到的反激励机制来约束 Sequencer 的行为，但如果 Sequencer 无视这种惩罚，也可能导致交易的执行结果的不确定。这是一种基于经济博弈的安全保证方法。
2. 如果 Executor 窜改交易的执行结果，也可能导致用户得到错误的结果。这种情况下，用户可以自己运行 Executor 来验证结果（未来会提供无状态的轻节点），或者通过多个 Executor 进行结果确认。这个风险和用户信任某个 L1 RPC 节点的风险类似。
3. Proposer 定时在 L1 上公布 Rooch 的 `state_root`，Executor 可以根据这个 `state_root` 来校验自己的状态。如果发现不一致，要么是 Executor 有问题，要么是 Proposer 有问题，双方可以通过 L1 的仲裁合约对状态进行仲裁，并将自己的状态回滚到正确的 `state_root`。如果是 Proposer 的错误，Proposer 会得到惩罚。
4. Rooch 的状态在 L1 上达到最终确定，需要等待一个挑战公示周期。未来，我们会结合 zk-proof 来压缩这个周期。
5. 软件的 Bug 本身也会带来状态的不确定性，这个需要时间来验证和修复，极端情况下，可能需要依赖社会共识来解决。

总结一下：

在 Rooch 中，应用和开发者可以认为交易是即时确定的，它的安全依赖于一套反激励机制，并且已经在区块链中广泛使用。当然，反激励机制的网络需要逐渐构建，用户和开发者都需要参与进来。