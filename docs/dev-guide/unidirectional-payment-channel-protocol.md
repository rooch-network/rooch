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

## II. 方案一：独立通道模型 (Dedicated Channel Model)

此方案是构建支付通道的基础，每个通道都是一个独立的、自包含的实体，拥有自己的资金托管。

### A. 核心思想

协议的核心是一个部署在 Rooch 链上的 Move 模块，它定义了一个名为 `PaymentChannel` 的核心对象（Object）。这个对象作为支付流的信任锚点和最终结算层，而大部分交易和状态更新则在链下进行。

为了从根本上解决多设备并发更新的状态同步难题，并与 Rooch 的 DID 身份层深度集成，我们引入“主-子通道”（Hub-and-Spoke）架构。

*   **主通道**：链上的 `PaymentChannel` 对象，负责托管**单个支付关系**的总资金，并作为最终结算的信任根。
*   **子通道**：完全在链下维护的逻辑通道，每个子通道对应一个设备/会话。它有自己独立的 `sub_nonce` 和累积金额 `sub_accumulated_amount`。

### B. 与 `rooch_framework::did` 的集成

此方案的核心在于将子通道的身份验证与 Rooch 的 DID 模块绑定。子通道不再由裸公钥标识，而是由支付方 DID 文档中的一个**验证方法 (Verification Method)** 来标识和授权。

**链上状态:**

```move
struct SubChannelState has store {
    last_claimed_amount: u256,
    last_confirmed_nonce: u64,
}

struct PaymentChannel<CoinType: store> has key {
    sender: address, // 支付方的 Rooch 账户地址
    receiver: address,
    balance: Balance<CoinType>, // 通道专属资金
    // key 是子通道的唯一标识符 (如 verification_method_id 的哈希)
    sub_channels: Table<vector<u8>, SubChannelState>,
    status: u8,
    cancellation_info: Option<CancellationInfo>,
}

struct CancellationInfo has store {
    initiated_time: u64, // 区块时间戳
    pending_amount: u256,
}
```

### C. 核心操作流程

#### 1. 开启支付流 (`open_stream`)
支付方为**每一个**收款方调用 `open_stream`，创建一个独立的 `PaymentChannel` 对象并存入资金。

#### 2. 链下更新 (流式支付)

*   **RAV 结构扩展**：链下凭证 `SubRAV` (Sub-channel RAV) 包含子通道及验证方法信息。
    ```rust
    // 扩展后的 RAV 结构 (链下定义)
    struct SubRAV {
        master_channel_id: ObjectID,
        // 不再是裸的 sub_channel_id，而是支付方 DID 的验证方法 ID
        // 例如 "did:rooch:0x...#keys-1"
        verification_method_id: String,
        sub_accumulated_amount: u256,
        sub_nonce: u64,
    }
    ```
*   **链下交互**：
    1.  支付方的某个设备（对应一个 `VerificationMethod`）与接收方进行交互。
    2.  接收方生成针对该 `verification_method_id` 的 `SubRAV`。
    3.  支付方使用该 `VerificationMethod` 对应的私钥对 `SubRAV` 进行签名，并将 `SubRAV` 和签名一起发送给接收方。
    *   **关键优势**：不同设备使用不同的 `VerificationMethod`，在各自的逻辑子通道上更新状态，互不干扰。

#### 3. 中途提款 (`claim_from_sub_channel`)
接收方可以随时调用 `claim_from_sub_channel` 函数，从**该通道专属的 `balance`** 中提取特定子通道的累积资金。

#### 4. 关闭与取消

*   **接收方关闭 (`close_channel`)**: 接收方可以提交最新的 `SubRAV` 立即关闭通道并结算。这适用于双方合作的场景。
*   **支付方取消 (`initiate_cancellation`)**: 支付方可以单方面发起取消请求，这会启动一个**挑战期**。

#### 5. 结算与仲裁

*   **争议解决 (`dispute`)**: 在挑战期内，如果接收方发现支付方提交了过时的 `SubRAV`，可以提交一个更新的 `SubRAV` 作为欺诈证明，以纠正最终的结算金额。
*   **最终结算 (`finalize_cancellation`)**: 挑战期结束后，任何人都可以调用此函数，根据最终确认的金额完成结算。

### D. Move 模块接口设计 (概念性)

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
    struct SubChannelState has store {
        last_claimed_amount: u256,
        last_confirmed_nonce: u64,
    }

    struct PaymentChannel<CoinType: store> has key {
        sender: address, // 支付方的 Rooch 账户地址
        receiver: address,
        balance: Balance<CoinType>, // 通道专属资金
        // key 是子通道的唯一标识符 (如 verification_method_id 的哈希)
        sub_channels: Table<vector<u8>, SubChannelState>,
        status: u8,
        cancellation_info: Option<CancellationInfo>,
    }

    struct CancellationInfo has store {
        initiated_time: u64, // 区块时间戳
        pending_amount: u256,
    }

    // === Public Functions for Scheme 1 ===

    public entry fun open_stream<CoinType: store>(
        sender: &signer,
        receiver: address,
        deposit: Coin<CoinType>
    ) { /* ... */ }

    public entry fun claim_from_sub_channel<CoinType: store>(
        receiver: &signer,
        channel_obj: &mut Object<PaymentChannel<CoinType>>,
        sender_vm_id_fragment: String,
        sub_accumulated_amount: u256,
        sub_nonce: u64,
        sender_signature: vector<u8>
    ) { /* ... */ }

    public entry fun close_channel<CoinType: store>(
        receiver: &signer,
        channel_obj: &mut Object<PaymentChannel<CoinType>>,
        sender_vm_id_fragment: String,
        sub_accumulated_amount: u256,
        sub_nonce: u64,
        sender_signature: vector<u8>
    ) { /* ... */ }

    public entry fun initiate_cancellation<CoinType: store>(
        sender: &signer,
        channel_obj: &mut Object<PaymentChannel<CoinType>>
    ) { /* ... */ }

    public entry fun dispute<CoinType: store>(
        receiver: &signer,
        channel_obj: &mut Object<PaymentChannel<CoinType>>,
        sender_vm_id_fragment: String,
        disputed_accumulated_amount: u256,
        disputed_nonce: u64,
        sender_signature: vector<u8>
    ) { /* ... */ }

    public entry fun finalize_cancellation<CoinType: store>(
        channel_obj: &mut Object<PaymentChannel<CoinType>>
    ) { /* ... */ }

    public entry fun top_up<CoinType: store>(
        sender: &signer,
        channel_obj: &mut Object<PaymentChannel<CoinType>>,
        additional_deposit: Coin<CoinType>
    ) { /* ... */ }

    // === Internal Helper Functions ===

    fun get_sub_rav_hash(
        master_channel_id: ObjectID,
        vm_id_fragment: &String,
        accumulated_amount: u256,
        nonce: u64
    ): vector<u8> {
        // ... (哈希内容应包含所有 SubRAV 字段)
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

        // 3. (核心) 检查该 VM 是否有权进行支付验证
        // 我们要求该 key 必须在 'authentication' 关系中
        let auth_methods = did::doc_authentication_methods(did_doc);
        let vm_id_string = did::format_verification_method_id(did::verification_method_id(&vm));
        assert!(vector::contains(auth_methods, &vm_id_string), EInsufficientPermission);

        // 4. 获取公钥并验证签名
        let public_key_multibase = did::verification_method_public_key_multibase(&vm);
        // 需要从 multibase 格式解码出公钥裸字节
        let (pk_bytes, _) = multibase_codec::decode(public_key_multibase);

        let msg_hash = get_sub_rav_hash(master_channel_id, vm_id_fragment, accumulated_amount, nonce);
        schnorr::verify(&pk_bytes, &msg_hash, signature)
    }
}
```

## III. 方案二：共享资金池模型 (Shared Balance Pool Model)

在方案一的基础上，为了解决支付方资金被多个独立通道“碎片化”占用的问题，我们设计了更高级的共享资金池模型。

### A. 核心思想

此方案将**资金托管**与**通道状态管理**彻底分离，允许同一个支付方的多个支付通道共享一个统一的资金池。
它复用了方案一中所有的链下机制，包括 `SubRAV` 的生成、签名和验证逻辑，其核心区别在于链上资金的组织方式。

1.  **`SharedBalancePool` 对象**: 一个由支付方创建并拥有的新对象，作为其所有支付行为的中央资金来源。
2.  **`LinkedPaymentChannel` 对象**: 新的通道对象，它自身**不持有资金**，而是通过 `ObjectID` 链接到一个 `SharedBalancePool`。

**架构优势**:
*   **高资金利用率**: 支付方无需为每个收款方单独锁定资金。一笔总资金可以支撑对多个收款方的并发微支付。
*   **简化管理**: 支付方只需管理一个总资金池，充值(`top_up`)操作也变得极为简单。
*   **风险提示**: 该模型引入了资金超额使用的风险。即，某个通道的提款请求可能会因为资金池已被其他通道提空而失败。但这对于高频、小额、可容忍失败的微支付场景是可接受的权衡。

### B. 链上状态

```move
// 新增：共享资金池对象
struct SharedBalancePool<CoinType: store> has key {
    owner: address, // 资金池的所有者 (即支付方)
    balance: Balance<CoinType>,
}

// 改造后的通道对象
struct LinkedPaymentChannel<CoinType: store> has key {
    sender: address,
    receiver: address,
    balance_pool_id: ObjectID, // 链接到共享资金池
    sub_channels: Table<vector<u8>, SubChannelState>,
    status: u8,
    cancellation_info: Option<CancellationInfo>,
}
```

### C. 核心操作变更

#### 1. 创建资金池与开启通道
*   **`create_balance_pool`**: 支付方首先调用此函数创建自己的 `SharedBalancePool` 并存入资金。
*   **`open_linked_channel`**: 之后，为每个收款方开启通道时，调用此函数，并传入 `SharedBalancePool` 的 `ObjectID`。

#### 2. 中途提款 (`claim_from_linked_channel`)
这是变更的核心。
*   **链上操作**:
    *   接收方调用 `claim_from_linked_channel`，提交 `LinkedPaymentChannel` 对象和 `SubRAV`。
*   **合约逻辑**:
    1.  从 `LinkedPaymentChannel` 对象中获取 `balance_pool_id`。
    2.  通过 `object::borrow_mut_by_id<SharedBalancePool<CoinType>>` 获取共享资金池的可变引用。
    3.  验证 `SubRAV` 的签名和 `nonce`。
    4.  计算增量金额。
    5.  **尝试从共享资金池的 `balance` 中提款**。如果资金不足，交易将失败。
    6.  更新 `LinkedPaymentChannel` 中该子通道的状态。

### D. Move 模块接口设计 (概念性)

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
    struct SharedBalancePool<CoinType: store> has key {
        owner: address, // 资金池的所有者 (即支付方)
        balance: Balance<CoinType>,
    }

    struct LinkedPaymentChannel<CoinType: store> has key {
        sender: address,
        receiver: address,
        balance_pool_id: ObjectID, // 链接到共享资金池
        sub_channels: Table<vector<u8>, SubChannelState>,
        status: u8,
        cancellation_info: Option<CancellationInfo>,
    }
    
    struct SubChannelState has store {
        last_claimed_amount: u256,
        last_confirmed_nonce: u64,
    }

    struct CancellationInfo has store {
        initiated_time: u64, // 区块时间戳
        pending_amount: u256,
    }

    // === Public Functions for Scheme 2 ===

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
    ) {
        // ...
    }

    /// 接收方从一个链接的子通道提款
    public entry fun claim_from_linked_channel<CoinType: store>(
        receiver: &signer,
        channel_obj: &mut Object<LinkedPaymentChannel<CoinType>>,
        balance_pool_obj: &mut Object<SharedBalancePool<CoinType>>, // 需要传入资金池对象
        sender_vm_id_fragment: String,
        sub_accumulated_amount: u256,
        sub_nonce: u64,
        sender_signature: vector<u8>
    ) {
        let channel = object::borrow_mut(channel_obj);
        assert!(signer::address_of(receiver) == channel.receiver, ENotReceiver);
        
        // 验证签名 (与方案一类似)
        // ...

        // 从共享资金池提款
        let pool = object::borrow_mut(balance_pool_obj);
        assert!(pool.owner == channel.sender, EInvalidBalancePool);

        // ... (计算增量金额)
        // let incremental_amount = ...
        // let coins_to_claim = balance::withdraw(&mut pool.balance, incremental_amount);
        // transfer::public_transfer(coins_to_claim, channel.receiver);

        // ... (更新子通道状态)
    }

    // ... (其他函数如 dispute, close_channel 等也需要相应调整)
}
```

## IV. 应用场景示例：AI 代理协议（以 Nuwa 为例）

为了更好地理解该支付协议的应用，我们以一个构建在 Rooch 上的 AI 代理协议（例如 Nuwa 协议）为例。在该场景中，一个客户端 AI 代理需要向一个提供 LLM 推理服务的网关代理支付费用。

1.  **开启支付流**：客户端代理调用 `open_stream`，存入 100 ROOCH 代币，为 LLM 网关创建一个支付通道。
2.  **链下流式支付**：
    *   客户端代理每次调用 LLM 网关的 API，网关都会计算费用，并生成一个新的 RAV。
    *   双方通过链下消息交换并双重签名该 RAV。例如，第一次调用后 `accumulated_amount` 为 0.5 ROOCH，第二次后为 1.2 ROOCH，`nonce` 依次递增。
3.  **关闭与结算**：
    *   **场景一 (接收方关闭)**：LLM 网关决定关闭通道，提交了 `nonce` 为 15，`accumulated_amount` 为 80 ROOCH 的最新双重签名 RAV。合约验证通过，立即将 80 ROOCH 转给网关，剩余 20 ROOCH 退还给客户端代理。
    *   **场景二 (支付方恶意取消)**：客户端代理调用 `initiate_cancellation`，但提交了一个 `nonce` 为 10，`accumulated_amount` 为 50 ROOCH 的过时 RAV。
    *   **争议**：在挑战期内，LLM 网关发现此行为，立即调用 `dispute` 函数，提交了 `nonce` 为 15，金额为 80 ROOCH 的最新 RAV 作为欺诈证明。
    *   **最终结算**：合约采纳了 LLM 网关的证明。挑战期结束后，`finalize_cancellation` 会将 80 ROOCH 结算给网关，剩余 20 ROOCH 退还给客户端代理。

## V. 结论与未来展望
本方案为 Rooch 网络设计了一套灵活、分层的单向状态通道流支付协议。
*   **独立通道模型**为基础的点对点支付提供了安全保障。
*   **共享资金池模型**通过解耦资金与通道状态，极大地提升了资金利用率和用户体验，特别适合需要同时与多个对手方进行高频微支付的复杂场景。

这为 Rooch 生态（如 Nuwa 协议）中的应用提供了从简单到高级的支付解决方案，开发者可以根据自身业务的特定需求和风险偏好来选择合适的模型。
