# Rooch 单向状态通道流支付协议

## I. 协议概述与核心原则

### A. 背景：Rooch 网络与微支付需求

Rooch 是一个高性能的模块化区块链网络，旨在为大规模去中心化应用提供基础设施。在 Rooch 生态中，许多应用场景，如 AI 代理间的服务调用、游戏内的道具交易、物联网设备间的数据交换等，都表现为高频、小额的“微支付”形态。

传统的链上交易因其固有的延迟和成本，难以高效支持此类微支付场景。为了解决这一挑战，我们设计了一套专为 Rooch 网络优化的单向状态通道流支付协议。此协议的核心思想是：将绝大多数支付状态的更新转移到链下处理，仅在通道开启、关闭和出现争议时与链上交互，从而在保证资金安全的前提下，最大化支付效率和灵活性。

### B. 核心优势

本方案旨在解决 Rooch 应用在微支付场景下的核心挑战，具备以下优势：

1.  **极致的性能与低成本**：通过将计费和支付确认过程完全置于链下，实现了近乎即时的支付体验，同时将链上交易成本降至最低。
2.  **灵活的计费模型**：将复杂的计费逻辑（如按 API 调用次数、按 Token 消耗量、动态定价等）完全移至链下，使服务提供方可以灵活调整其商业模式，而无需修改链上合约。
3.  **与 Rooch 原生账户集成**：协议直接利用 Rooch 的原生账户体系及其 DID 模型，包括其多密钥管理功能，为多设备、多会话场景提供了优雅的解决方案。
4.  **异步与非合作安全性**：通过引入挑战期和欺诈证明机制，协议有效防止了恶意行为，并降低了双方持续在线监控的负担，同时支持通过“瞭望塔”服务实现委托监控。

## II. 技术方案：基于共享资金池与 DID 子通道的架构

本协议的核心设计思想是将**资金托管**与**通道状态管理**彻底分离，并利用 Rooch 的原生 DID 功能来管理多设备权限，从而实现一个既高效又灵活的支付系统。

### A. 核心架构

该架构由三个关键组件构成：

1.  **`SharedBalancePool` (共享资金池)**: 一个由支付方创建并拥有的独立对象，作为其所有支付行为的中央资金来源。这解决了资金被多个独立通道“碎片化”占用的问题。
2.  **`LinkedPaymentChannel` (链接支付通道)**: 一个轻量级的通道状态对象。它自身**不直接持有资金**，而是通过 `ObjectID` 链接到一个 `SharedBalancePool`。每个支付关系（支付方 -> 收款方）都会创建一个独立的 `LinkedPaymentChannel`。
3.  **`Sub-channel` (子通道)**: 一个完全在链下维护的逻辑概念，与支付方的一个特定**验证方法 (Verification Method)** 绑定，用于处理来自单个设备或会话的并发支付流。每个子通道都有自己独立的 `nonce` 和累积金额。

**架构优势**:
*   **高资金利用率**: 支付方无需为每个收款方单独锁定资金。一笔总资金可以支撑对多个收款方的并发微支付。
*   **简化管理**: 支付方只需管理一个总资金池，充值(`top_up`)操作也变得极为简单。
*   **原生多设备支持**: 通过将子通道与 DID 的 `VerificationMethod` 绑定，完美解决了多设备的状态同步和授权问题。
*   **风险提示**: 该模型引入了资金超额使用的风险。即，某个通道的提款请求可能会因为资金池已被其他通道提空而失败。但这对于高频、小额、可容忍失败的微支付场景是可接受的权衡。

### B. 链上状态定义

```move
// 共享资金池对象
struct SharedBalancePool<CoinType: store> has key {
    owner: address, // 资金池的所有者 (即支付方)
    balance: Balance<CoinType>,
}

// 链接支付通道对象
struct LinkedPaymentChannel<CoinType: store> has key {
    sender: address,
    receiver: address,
    balance_pool_id: ObjectID, // 链接到共享资金池
    sub_channels: Table<vector<u8>, SubChannelState>,
    status: u8, // 0: Active, 1: Cancelling, 2: Closed
    cancellation_info: Option<CancellationInfo>,
}

// 子通道的链上状态记录
struct SubChannelState has store {
    last_claimed_amount: u256,
    last_confirmed_nonce: u64,
}

// 用于支付方单方面取消通道时的状态记录
struct CancellationInfo has store {
    initiated_time: u64, // 区块时间戳
    pending_amount: u256,
}
```

### C. 核心操作流程

#### 1. 设置阶段

*   **`create_balance_pool`**: 支付方首先调用此函数，创建自己的 `SharedBalancePool` 并存入初始资金。
*   **`open_linked_channel`**: 之后，支付方为每一个收款方调用此函数，传入 `SharedBalancePool` 的 `ObjectID`，创建一个链接通道。

#### 2. 链下流式支付 (基于子通道)

*   **`SubRAV` 结构**: 链下凭证 `SubRAV` (Sub-channel RAV) 是所有链下交互的核心。
    ```rust
    // 扩展后的 RAV 结构 (链下定义)
    struct SubRAV {
        master_channel_id: ObjectID, // 这里指 LinkedPaymentChannel 的 ID
        verification_method_id: String, // 支付方 DID 的验证方法 ID
        sub_accumulated_amount: u256,
        sub_nonce: u64,
    }
    ```
*   **链下交互**: 与“独立通道模型”完全一致。支付方的设备使用其对应的 `VerificationMethod` 私钥对 `SubRAV` 进行签名，并与接收方进行高频的状态更新，互不干扰。

#### 3. 中途提款 (`claim_from_linked_channel`)

这是该架构的核心优势体现。接收方可以在不关闭通道的情况下，随时提取任何一个子通道的累积资金。

*   **链上操作**: 接收方调用 `claim_from_linked_channel`，提交 `LinkedPaymentChannel` 的对象引用、`SharedBalancePool` 的对象引用，以及最新的 `SubRAV` 和签名。
*   **合约逻辑**:
    1.  验证 `SubRAV` 的签名和 `nonce`，确保其有效且未被使用过。
    2.  计算增量金额：`incremental_amount = sub_accumulated_amount - sub_channel_state.last_claimed_amount`。
    3.  **从共享资金池 `SharedBalancePool` 的 `balance` 中，将 `incremental_amount` 对应的代币转给接收方。**
    4.  更新 `LinkedPaymentChannel` 中该子通道的链上状态。

#### 4. 关闭、取消与仲裁

通道的关闭和争议解决流程与基础模型类似，但所有资金操作都将指向共享资金池。

*   **合作关闭 (`close_channel`)**: 接收方提交最新 `SubRAV`，合约验证后，从共享资金池结算最后一笔款项，并将 `LinkedPaymentChannel` 标记为 `Closed`。
*   **单方面取消 (`initiate_cancellation` & `dispute` & `finalize_cancellation`)**: 支付方可以单方面发起取消，进入挑战期。接收方可以在挑战期内提交更新的 `SubRAV` 进行争议。挑战期结束后，最终结算的资金同样来自共享资金池。

## III. Move 模块接口设计 (概念性)

```move
// file: sources/streaming_payment.move
module rooch_examples::streaming_payment {
    use std::option::{Self, Option};
    use std::signer;
    use rooch_framework::object::{Self, Object, ObjectID};
    use rooch_framework::balance::{Self, Balance};
    use rooch_framework::coin::Coin;
    use rooch_framework::crypto::schnorr;
    use rooch_framework::timestamp;
    use rooch_framework::transfer;
    use moveos_std::type_info;
    use moveos_std::bcs;
    use moveos_std::u256;
    use rooch_framework::did;
    use moveos_std::string::String;
    use moveos_std::table::{Self, Table};
    use moveos_std::multibase_codec;
    use std::vector;

    // === Constants ===
    const STATUS_ACTIVE: u8 = 0;
    const STATUS_CANCELLING: u8 = 1;
    const STATUS_CLOSED: u8 = 2;
    const CHALLENGE_PERIOD_SECONDS: u64 = 86400; // 1 day

    // === Structs ===
    // SharedBalancePool, LinkedPaymentChannel, SubChannelState, CancellationInfo
    // as defined in the section above.

    // === Public Functions ===

    /// 支付方创建并初始化其共享资金池
    public entry fun create_balance_pool<CoinType: store>(
        sender: &signer,
        initial_deposit: Coin<CoinType>
    ) {
        let pool = SharedBalancePool {
            owner: signer::address_of(sender),
            balance: balance::new(initial_deposit),
        };
        // 将对象所有权转移给 sender
        transfer::public_transfer(object::new(pool), signer::address_of(sender));
    }

    /// 支付方为某个收款方开启一个链接到资金池的通道
    public entry fun open_linked_channel<CoinType: store>(
        sender: &signer,
        receiver: address,
        balance_pool_id: ObjectID
    ) { /* ... */ }

    /// 接收方从一个链接的子通道提款
    public entry fun claim_from_linked_channel<CoinType: store>(
        receiver: &signer,
        channel_obj: &mut Object<LinkedPaymentChannel<CoinType>>,
        balance_pool_obj: &mut Object<SharedBalancePool<CoinType>>,
        sender_vm_id_fragment: String,
        sub_accumulated_amount: u256,
        sub_nonce: u64,
        sender_signature: vector<u8>
    ) { /* ... */ }

    // ... (其他函数如 top_up_pool, close_channel, initiate_cancellation, dispute, finalize_cancellation)

    // === Internal Helper Functions ===

    fun get_sub_rav_hash(
        master_channel_id: ObjectID,
        vm_id_fragment: &String,
        accumulated_amount: u256,
        nonce: u64
    ): vector<u8> {
        bcs::to_bytes(&(master_channel_id, vm_id_fragment, accumulated_amount, nonce))
    }

    fun verify_sender_signature(
        master_channel_id: ObjectID,
        sender_address: address,
        vm_id_fragment: &String,
        accumulated_amount: u256,
        nonce: u64,
        signature: &vector<u8>
    ): bool {
        // 1. 获取支付方的 DID Document
        let did_doc = did::get_did_document_by_address(sender_address);

        // 2. 从 DID Document 中查找对应的 Verification Method
        let vm_option = did::doc_verification_method(did_doc, vm_id_fragment);
        assert!(option::is_some(&vm_option), EVerificationMethodNotFound);
        let vm = option::destroy_some(vm_option);

        // 3. (核心) 检查该 VM 是否有权进行支付验证 (authentication)
        let auth_methods = did::doc_authentication_methods(did_doc);
        let vm_id_string = did::format_verification_method_id(did::verification_method_id(&vm));
        assert!(vector::contains(auth_methods, &vm_id_string), EInsufficientPermission);

        // 4. 获取公钥并验证签名
        let public_key_multibase = did::verification_method_public_key_multibase(&vm);
        let (pk_bytes, _) = multibase_codec::decode(public_key_multibase);

        let msg_hash = get_sub_rav_hash(master_channel_id, vm_id_fragment, accumulated_amount, nonce);
        schnorr::verify(&pk_bytes, &msg_hash, signature)
    }
}
```

## IV. 应用场景示例：AI 代理协议（以 Nuwa 为例）

为了更好地理解该支付协议的应用，我们以一个构建在 Rooch 上的 AI 代理协议（例如 Nuwa 协议）为例。在该场景中，一个客户端 AI 代理需要向多个不同的服务（如 LLM 推理、数据存储、图像生成）支付费用。

1.  **一次性设置资金池**: 客户端代理调用 `create_balance_pool`，存入 1000 ROOCH 代币作为其所有微支付的总预算。
2.  **开启多个链接通道**: 
    *   客户端为 LLM 网关调用 `open_linked_channel`，链接到它的资金池。
    *   客户端为数据存储服务调用 `open_linked_channel`，也链接到同一个资金池。
3.  **并发链下支付**: 
    *   客户端代理在手机上调用 LLM API，使用手机的密钥（对应一个 `VerificationMethod`）对 `SubRAV` 签名。
    *   同时，其在笔记本电脑上的脚本自动备份数据，使用笔记本的密钥（对应另一个 `VerificationMethod`）对发往存储服务的 `SubRAV` 进行签名。
4.  **独立提款**: 
    *   LLM 网关在累积了 50 ROOCH 的费用后，调用 `claim_from_linked_channel` 从共享资金池中提款。
    *   数据存储服务在累积了 10 ROOCH 后，也独立调用 `claim_from_linked_channel` 提款。

这个流程充分展示了该统一方案的灵活性和高资金效率。

## V. 结论与未来展望

本方案为 Rooch 网络设计了一套统一且强大的单向状态通道流支付协议。通过将**共享资金池**与 **DID 子通道**无缝结合，该方案不仅解决了基础的微支付需求，还优雅地处理了资金碎片化和多设备并发的复杂问题，为开发者提供了强大而灵活的支付基础设施。

这为 Rooch 生态（如 Nuwa 协议）中的应用提供了“一步到位”的支付解决方案，能够支撑从简单到复杂的各类业务场景。
