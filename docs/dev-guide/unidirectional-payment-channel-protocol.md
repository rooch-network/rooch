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

1.  **`PaymentHub` (支付中心)**: 一个由支付方创建并拥有的、非泛型的独立对象。它内部包含一个 `MultiCoinStore` 对象的 ID，使其能统一管理**多种不同类型**的代币。这不仅解决了资金碎片化问题，还为协议添加了一个可扩展的策略层（如设置通道配额）。
2.  **`PaymentChannel<CoinType>` (支付通道)**: 一个轻量级的、**泛型**的通道状态对象。它自身不直接持有资金，而是通过 `ObjectID` 链接到一个 `PaymentHub`。每个通道都严格绑定一种 `CoinType`，确保了类型安全。
3.  **`Sub-channel` (子通道)**: 一个完全在链下维护的逻辑概念，与支付方的一个特定**验证方法 (Verification Method)** 绑定，用于处理来自单个设备或会话的并发支付流。每个子通道都有自己独立的 `nonce` 和累积金额。

**架构优势**:
*   **通用支付中心**: 用户只需管理一个 `PaymentHub`，即可支持所有币种的支付通道，极大简化了用户操作。
*   **协议可扩展性**: 可以在 `PaymentHub` 上添加协议级别的策略和限制（如最大通道数、总流出速率等），而无需修改标准的 `MultiCoinStore`。
*   **类型安全**: `PaymentChannel<CoinType>` 的泛型设计确保了每个通道的账本和链下凭证在编译时就是类型安全的。
*   **原生多设备支持**: 通过将子通道与 DID 的 `VerificationMethod` 绑定，完美解决了多设备的状态同步和授权问题。

### B. 链上状态定义

```move
// 支付中心对象 (非泛型)
struct PaymentHub has key {
    owner: address, // 支付中心的所有者 (即支付方)
    multi_coin_store_id: ObjectID, // 链接到 MultiCoinStore 对象
    // 未来可扩展的策略字段，例如：
    // max_open_channels: u64,
    // total_outflow_limit: u256,
}

// 支付通道对象 (泛型)
struct PaymentChannel<CoinType: store> has key {
    sender: address,
    receiver: address,
    payment_hub_id: ObjectID, // 链接到 PaymentHub 对象
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

*   **`create_payment_hub`**: 支付方首次调用此函数，创建一个 `PaymentHub`。此函数会自动创建一个关联的 `MultiCoinStore` 并将其 ID 存储在中心内。
*   **`deposit_to_hub<CoinType>`**: 支付方调用此函数，将特定类型和数量的 `Coin<CoinType>` 存入其 `PaymentHub` 中。
*   **`open_channel<CoinType>`**: 之后，支付方为每一个收款方调用此函数，传入 `PaymentHub` 的 `ObjectID` 和 `CoinType`，创建一个特定币种的通道。

#### 2. 链下流式支付 (基于子通道)

*   **`SubRAV` 结构**: 链下凭证 `SubRAV` (Sub-channel RAV) 是所有链下交互的核心。
    ```rust
    // 链下 RAV 结构定义
    struct SubRAV {
        channel_id: ObjectID, // PaymentChannel 的 ObjectID
        verification_method_id: String, // 支付方 DID 的验证方法 ID
        sub_accumulated_amount: u256,
        sub_nonce: u64,
    }
    ```
*   **链下交互**: 与“独立通道模型”完全一致。支付方的设备使用其对应的 `VerificationMethod` 私钥对 `SubRAV` 进行签名，并与接收方进行高频的状态更新，互不干扰。

#### 3. 中途提款 (`claim_from_channel`)

这是该架构的核心优势体现。接收方可以在不关闭通道的情况下，随时提取任何一个子通道的累积资金。

*   **链上操作**: 接收方调用 `claim_from_channel<CoinType>`，提交 `PaymentChannel` 的对象引用、`PaymentHub` 的对象引用，以及最新的 `SubRAV` 和签名。
*   **合约逻辑**:
    1.  验证 `SubRAV` 的签名和 `nonce`。
    2.  从 `PaymentChannel` 获取 `payment_hub_id`，并借出 `PaymentHub` 对象。
    3.  从 `PaymentHub` 对象中获取 `multi_coin_store_id`。
    4.  计算增量金额：`incremental_amount = sub_accumulated_amount - sub_channel_state.last_claimed_amount`。
    5.  **从 `MultiCoinStore` 中，使用 `multi_coin_store::withdraw` 并传入 `CoinType` 的类型信息，提取 `incremental_amount` 对应的代币，然后转给接收方。**
    6.  更新 `PaymentChannel` 中该子通道的链上状态。

#### 4. 关闭、取消与仲裁

通道的关闭和争议解决流程与基础模型类似，但所有资金操作都将指向支付中心。

*   **合作关闭 (`close_channel`)**: 接收方提交最新 `SubRAV`，合约验证后，从支付中心结算最后一笔款项，并将 `PaymentChannel` 标记为 `Closed`。
*   **单方面取消 (`initiate_cancellation` & `dispute` & `finalize_cancellation`)**: 支付方可以单方面发起取消，进入挑战期。接收方可以在挑战期内提交更新的 `SubRAV` 进行争议。挑战期结束后，最终结算的资金同样来自支付中心。

## III. Move 模块接口设计 (概念性)

```move
// file: sources/payment_channel.move
module rooch_framework::payment_channel {
    use std::option::{Self, Option};
    use std::signer;
    use std::vector;

    use moveos_std::bcs;
    use moveos_std::context;
    use moveos_std::object::{Self, Object, ObjectID};
    use moveos_std::string::{Self, String};
    use moveos_std::table::{Self, Table};
    use moveos_std::type_info;
    use moveos_std::u256;

    use rooch_framework::account_authentication;
    use rooch_framework::coin::{Self, Coin, GenericCoin};
    use rooch_framework::multi_coin_store::{Self, MultiCoinStore};
    use rooch_framework::did;
    use rooch_framework::timestamp;
    use rooch_framework::transfer;

    // === Error Constants ===
    /// The signer is not the designated receiver of the channel.
    const ErrorNotReceiver: u64 = 1;
    /// The channel is not in an active state.
    const ErrorChannelNotActive: u64 = 2;
    /// The provided signature from the sender is invalid.
    const ErrorInvalidSenderSignature: u64 = 3;
    /// The specified Verification Method was not found in the sender's DID.
    const ErrorVerificationMethodNotFound: u64 = 4;
    /// The Verification Method used does not have 'authentication' permission.
    const ErrorInsufficientPermission: u64 = 5;
    /// The provided payment hub object does not match the one linked in the channel.
    const ErrorInvalidPaymentHub: u64 = 6;
    /// The nonce for the sub-channel is not greater than the last confirmed nonce.
    const ErrorInvalidNonce: u64 = 7;
    /// The claimed amount is less than or equal to the already claimed amount.
    const ErrorInvalidAmount: u64 = 8;
    /// The owner of the payment hub does not match the sender of the channel.
    const ErrorHubOwnerMismatch: u64 = 9;


    // === Constants ===
    const STATUS_ACTIVE: u8 = 0;
    const STATUS_CANCELLING: u8 = 1;
    const STATUS_CLOSED: u8 = 2;
    const CHALLENGE_PERIOD_SECONDS: u64 = 86400; // 1 day

    // === Structs ===
    /// A central, user-owned object for managing payments.
    /// It contains a MultiCoinStore to support various coin types.
    struct PaymentHub has key {
        owner: address,
        multi_coin_store_id: ObjectID,
    }

    /// A lightweight object representing a payment relationship, linked to a PaymentHub.
    struct PaymentChannel<CoinType: store> has key {
        sender: address,
        receiver: address,
        payment_hub_id: ObjectID, // Links to a PaymentHub object
        sub_channels: Table<vector<u8>, SubChannelState>,
        status: u8,
        cancellation_info: Option<CancellationInfo>,
    }
    
    /// The on-chain state for a specific sub-channel.
    struct SubChannelState has store {
        last_claimed_amount: u256,
        last_confirmed_nonce: u64,
    }

    /// Information stored when a channel cancellation is initiated.
    struct CancellationInfo has store {
        initiated_time: u64,
        pending_amount: u256,
    }

    // === Public Functions ===

    /// Creates and initializes a payment hub for the sender.
    /// This also creates an associated MultiCoinStore.
    public entry fun create_payment_hub() {
        let sender = context::sender();
        let multi_coin_store_id = multi_coin_store::create_multi_coin_store(sender);
        let hub = PaymentHub {
            owner: sender,
            multi_coin_store_id,
        };
        transfer::transfer(hub, sender);
    }

    /// Deposits a specific type of coin into the payment hub.
    public entry fun deposit_to_hub<CoinType: key + store>(
        hub_id: ObjectID,
        coin: Coin<CoinType>,
    ) {
        let sender = context::sender();
        let hub_obj = object::borrow_object<PaymentHub>(hub_id);
        let hub = object::borrow(hub_obj);
        assert!(hub.owner == sender, ErrorHubOwnerMismatch);

        let multi_coin_store_obj = multi_coin_store::borrow_mut_coin_store_internal(hub.multi_coin_store_id);
        let generic_coin = coin::convert_coin_to_generic_coin(coin);
        multi_coin_store::deposit(multi_coin_store_obj, generic_coin);
    }

    /// Opens a new payment channel linked to a payment hub.
    public entry fun open_channel<CoinType: key + store>(
        receiver: address,
        payment_hub_id: ObjectID
    ) {
        let sender = context::sender();
        
        // Ensure the sender owns the payment hub
        let hub_obj = object::borrow_object<PaymentHub>(payment_hub_id);
        let hub = object::borrow(hub_obj);
        assert!(hub.owner == sender, ErrorHubOwnerMismatch);

        let channel = PaymentChannel<CoinType> {
            sender,
            receiver,
            payment_hub_id,
            sub_channels: table::new(),
            status: STATUS_ACTIVE,
            cancellation_info: option::none(),
        };
        transfer::transfer(channel, sender);
    }

    /// The receiver claims funds from a specific sub-channel.
    public entry fun claim_from_channel<CoinType: key + store>(
        channel_id: ObjectID, // The signer must be the receiver.
        payment_hub_id: ObjectID,
        sender_vm_id_fragment: vector<u8>,
        sub_accumulated_amount: u256,
        sub_nonce: u64,
        sender_signature: vector<u8>
    ) {
        let channel_obj = object::borrow_mut_object_extend<PaymentChannel<CoinType>>(channel_id);
        let channel = object::borrow_mut(channel_obj);

        // The transaction sender must be the receiver.
        assert!(channel.status == STATUS_ACTIVE, ErrorChannelNotActive);

        // Verify that the correct payment hub is being used.
        assert!(channel.payment_hub_id == payment_hub_id, ErrorInvalidPaymentHub);

        // Verify the sender's signature on the off-chain proof (SubRAV).
        assert!(
            verify_sender_signature(
                channel_id,
                channel.sender,
                &sender_vm_id_fragment,
                sub_accumulated_amount,
                sub_nonce,
                &sender_signature
            ),
            ErrorInvalidSenderSignature
        );
        
        // Get or create the sub-channel state.
        let sub_channel_state = if (table::contains(&channel.sub_channels, sender_vm_id_fragment)) {
            table::borrow_mut(&mut channel.sub_channels, sender_vm_id_fragment)
        } else {
            table::add(&mut channel.sub_channels, sender_vm_id_fragment, SubChannelState {
                last_claimed_amount: u256::zero(),
                last_confirmed_nonce: 0,
            });
            table::borrow_mut(&mut channel.sub_channels, sender_vm_id_fragment)
        };

        // Validate amount and nonce are strictly increasing.
        assert!(sub_accumulated_amount > sub_channel_state.last_claimed_amount, ErrorInvalidAmount);
        assert!(sub_nonce > sub_channel_state.last_confirmed_nonce, ErrorInvalidNonce);

        let incremental_amount = u256::sub(sub_accumulated_amount, sub_channel_state.last_claimed_amount);

        // Update the sub-channel state on-chain.
        sub_channel_state.last_claimed_amount = sub_accumulated_amount;
        sub_channel_state.last_confirmed_nonce = sub_nonce;
        
        // Withdraw funds from the payment hub and transfer to the receiver.
        let hub_obj = object::borrow_object<PaymentHub>(payment_hub_id);
        let hub = object::borrow(hub_obj);
        let multi_coin_store_obj = multi_coin_store::borrow_mut_coin_store_internal(hub.multi_coin_store_id);
        
        let coin_type_name = type_info::type_name<CoinType>();
        let generic_payment = multi_coin_store::withdraw(multi_coin_store_obj, string::to_string(coin_type_name), incremental_amount);
        let payment = coin::from_generic_coin<CoinType>(generic_payment);

        transfer::public_transfer(payment, channel.receiver);
    }

    // === Internal Helper Functions ===

    fun get_sub_rav_hash(
        channel_id: ObjectID,
        vm_id_fragment: &vector<u8>,
        accumulated_amount: u256,
        nonce: u64
    ): vector<u8> {
        bcs::to_bytes(&(channel_id, vm_id_fragment, accumulated_amount, nonce))
    }

    fun verify_sender_signature(
        channel_id: ObjectID,
        sender_address: address,
        vm_id_fragment: &vector<u8>,
        accumulated_amount: u256,
        nonce: u64,
        signature: &vector<u8>
    ): bool {
        let msg_hash = get_sub_rav_hash(channel_id, vm_id_fragment, accumulated_amount, nonce);
        
        // Construct the full verification method ID.
        let did_id = did::did_id(sender_address);
        let vm_id = did::verification_method_id(&did_id, vm_id_fragment);

        // Check if the VM has 'authentication' permission.
        assert!(
            did::has_permission_for_authentication(&did_id, &vm_id), 
            ErrorInsufficientPermission
        );

        // Verify the signature using the appropriate authentication key from the DID.
        account_authentication::verify_signature(&vm_id, msg_hash, signature)
    }
}
```

## IV. 应用场景示例：AI 代理协议（以 Nuwa 为例）

为了更好地理解该支付协议的应用，我们以一个构建在 Rooch 上的 AI 代理协议（例如 Nuwa 协议）为例。在该场景中，一个客户端 AI 代理需要向多个不同的服务（如 LLM 推理、数据存储、图像生成）支付费用。

1.  **一次性设置支付中心**: 客户端代理调用 `create_payment_hub`，并调用 `deposit_to_hub` 存入 1000 RGas 代币作为其所有微支付的总预算。
2.  **开启多个链接通道**: 
    *   客户端为 LLM 网关调用 `open_channel`，链接到它的支付中心。
    *   客户端为数据存储服务调用 `open_channel`，也链接到同一个支付中心。
3.  **并发链下支付**: 
    *   客户端代理在手机上调用 LLM API，使用手机的密钥（对应一个 `VerificationMethod`）对 `SubRAV` 签名。
    *   同时，其在笔记本电脑上的脚本自动备份数据，使用笔记本的密钥（对应另一个 `VerificationMethod`）对发往存储服务的 `SubRAV` 进行签名。
4.  **独立提款**: 
    *   LLM 网关在累积了 50 RGas 的费用后，调用 `claim_from_channel` 从支付中心中提款。
    *   数据存储服务在累积了 10 RGas 后，也独立调用 `claim_from_channel` 提款。

这个流程充分展示了该统一方案的灵活性和高资金效率。

## V. 结论与未来展望

本方案为 Rooch 网络设计了一套统一且强大的单向状态通道流支付协议。通过将**支付中心 (PaymentHub)** 与 **DID 子通道**无缝结合，该方案不仅解决了基础的微支付需求，还优雅地处理了资金碎片化和多设备并发的复杂问题，为开发者提供了强大而灵活的支付基础设施。

这为 Rooch 生态（如 Nuwa 协议）中的应用提供了“一步到位”的支付解决方案，能够支撑从简单到复杂的各类业务场景。

### C. 探索更高级的密码学原语

为了进一步增强协议的能力和应用范围，未来可以探索集成更高级的密码学技术：

*   **条件支付与可锁签名 (Conditional Payments & Lockable Signatures)**: 虽然在当前的双边单向模型中不是必需的，但可锁签名是实现网络化支付（如 A->B->C 多跳支付）和原子互换的关键。通过引入基于哈希时间锁（HTLC）的逻辑，可以将该协议从一个双边工具升级为一个网络化的支付基础设施，极大地扩展其应用场景。

*   **哈希链与支付计量 (Hash Chains & Payment Metering)**: 作为当前 `u64` nonce 的一种替代方案，哈希链可以为按次计费的场景提供更精细的支付计量。支付方可以预先生成一条哈希链，每次支付消费链上的一个哈希，为服务提供一种“即用即付”的精确凭证。虽然这会增加链下管理的复杂性，但为特定应用场景提供了额外的灵活性。
