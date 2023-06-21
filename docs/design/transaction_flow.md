# Transaction Flow

The main objective of this document is to explain the transaction processing flow in Rooch, in order to help DApp developers and Rooch developers to gain a deeper understanding of the design and implementation of Rooch, and thus participate more easily in the development of DApp and Rooch. At the same time, this document also attempts to answer some common questions, such as the execution order of transactions, transaction finality, and so on.


## User Perspective


From the user's perspective of calling, the transaction execution process in Rooch is as follows:

![rooch transaction flow user perspective](../static/design/rooch-design-transaction-flow-user-perspective.svg)

1. Users send transactions to Rooch RPC API via SDK or CLI. In addition, Rooch also supports Ethereum RPC API, so users can also send transactions to Rooch via clients that support Ethereum RPC, such as MetaMask. In the future, Rooch will support more L1's RPC APIs.
2. After receiving the transactions, Rooch's various RPC APIs send them to RPC Service for processing.
3. RPC Service will call modules such as Executor, Sequencer, Proposer, etc. to process the transactions.
4. The Sequencer and Proposer will interact with the backend L1 periodically in batches.


## Functional Perspective

From the functional perspective of internal components, the transaction execution process in Rooch is as follows:

![rooch transaction flow functional perspective](../static/design/rooch-design-transaction-flow-functional-perspective.svg)

1. After receiving transactions from different APIs, RPC Service first sends the multi-chain transactions to Executor for `validate_tx`.
2. In Executor's `validate_tx`:
    * The `rooch_framework::address_mapping::resolve` method is called first to convert the multi-chain address to Rooch address (Move address).
    * Then call the `rooch_framework::transaction_validator::validate` method to verify the transaction's `Authenticator`. The `Authenticator` represents the `sender`'s proof of identity, usually a signature. The `scheme` field in the `Authenticator` is used to indicate the type of the `Authenticator`. Currently, two built-in `Authenticators`, `ED25519` and Ethereum's `ECDSA`, are supported, and more signature types will be supported in the future, as well as allowing developers to customize `Authenticator`. This is also part of AccountAbstraction. In addition, the `sequence_number`(equivalent to `nonce` in Ethereum) of the account will also be validated in `validate`.
    * After validated by the contract, Executor constructs `TxContext` based on the transaction, uniformly converts the multi-chain transaction into `MoveOSTransaction`, and returns it.
3. After receiving the return value of `validate_tx` from Executor, RPC Service decide whether the transaction has passed verification. If not, an error is returned directly to the user. Otherwise, the transaction is sent to Sequencer for `sequence_tx`. Because `validate_tx` is a read-only method and does not modify the state, returning an error at this step has no side effects.
4. Sequencer adds the transaction to the Accumulator, obtains the `tx_order` of the transaction, and constructs `TransactionSequenceInfo`. `TransactionSequenceInfo` contains the signature of the `tx_order` assigned to the transaction by Sequencer and `tx_accumulator_root`. Sequencer periodically submits the transactions to DA in batches.
5. After RPC Service receives the return value of `sequence_tx` from Sequencer, it sends the transaction to Executor for `execute_tx`.
6. In Executor's `execute_tx`, MoveOS executes the transaction directly.
    * First, MoveOS executes the `rooch_framework::transaction_validator::pre_execute` method for preprocessing the transaction. In the preprocessing, an account is automatically created and multi-chain addresses are mapped to Move addresses. In the future, Gas-related requirements in AccountAbstraction will also be implemented in the preprocessing, such as Gas exchange and Gas payment agents.
    * Then, MoveOS calls the user-defined methods to execute the transaction.
    * Finally, MoveOS executes the `rooch_framework::transaction_validator::post_execute` method for post-processing the transaction. In the post-processing, the account's `sequence_number` is updated automatically and Gas fees are deducted.
    * During the execution process, `pre_execute`, `execute`, and `post_execute` share a `TxContext`, which can be used to pass data between them.
    * Note that if the user-defined method fails during execution, MoveOS will automatically roll back the state, but `pre_execute` and `post_execute` will still be executed, and Gas fees will only be charged for the actual consumption of the user's execution.
7. RPC Service sends the transaction to Proposer for `propose_tx` after receiving `execute_tx`'s return value, and Proposer packs the transaction into a block and periodically submits the block to L1's StateCommitment contract. Note that the block here does not contain the original data of the transaction, similar to the block header, it contains Rooch's `state_root` and the transaction's `tx_accumulator_root`.
8. Finally, RPC Service returns `TransactionSequenceInfo` and `TransactionExecutionInfo` to the user to represent a successful and confirmed transaction execution.

**Note:**
1. The logic of Challenger and fraud-proof, zk-proof is not included in the current flow. This part will be updated in future versions.
2. The entire process includes multiple components such as `Executor`,`Sequencer`,`Proposer`, but these components may not be on the same node, and they may communicate remotely through P2P network. This part will also be updated in future versions.
3. The above process is described based on the current design and some logic has not yet been fully implemented, and it will continue to be updated in the future.

## FAQ

### How is the execution order of transactions determined?

The execution order of transactions is determined by the Sequencer. The Sequencer adds transactions to the Accumulator, obtains the global order of the transaction immediately and signs the order of the transaction, which is a promise to the user that it will not modify the order or discard the transaction. The Accumulator can provide a proof of the order of the transaction. If the order of the transaction submitted by the Sequencer to DA is inconsistent with the previous commitment, the user can prove that the behavior of the Sequencer is malicious and punish the Sequencer.

### How is the execution result of a transaction determined?

There is no transaction pool in Rooch, and the execution result of a transaction is determined immediately. After the client submits the transaction, the result is immediately obtained without waiting for asynchronous consensus confirmation. Because if the execution order of the transaction is determined, the program is determined, and the execution result of the transaction is also determined. There are some security assumptions here, because the security assumptions of L2 are built on the basis of [counterfactual](https://en.wikipedia.org/wiki/Counterfactual_thinking) causal reasoning. If cheating behavior is will to be punished, the rational choice is not to cheat.

1. The execution order of transactions is ensured by the Sequencer, and users use the aforementioned counter-incentive mechanism to restrict the behavior of Sequencer. However, if the Sequencer ignores this punishment, it may also cause uncertainty in the execution result of the transaction. This is a security guarantee method based on economic game theory.
2. If Executor modifies the execution result of the transaction, it may also cause the user to obtain incorrect results. In this case, the user can run Executor to verify the result (a stateless light node will be provided in the future) or confirm the results through multiple Executors. This risk is similar to the risk of trusting a certain L1 RPC node.
3. Proposer periodically announces Rooch's `state_root` on L1, and Executor can verify its own state based on this `state_root`. If an inconsistency is found, it may be a problem with the Executor or with the Proposer. The two parties can arbitrate the state through L1's arbitration contract and roll back their own state to the correct `state_root`. If it is Proposer's error, Proposer will be punished.
4. Rooch's state reaches final determination on L1 and requires a challenge period. In the future, we will combine zk-proof to compress this period.
5. Software bugs themselves can also bring about uncertainty in the state, which takes time to verify and fix. In extreme cases, it may be necessary to rely on social consensus to resolve it.

To summarize:

In Rooch, applications and developers can assume that transactions are immediately determined, and their security depends on a set of counter-incentive mechanisms that have been widely used in blockchain. Of course, the network of counter-incentive mechanisms needs to be gradually constructed, with users and developers participating in it.