# Rooch 单向状态通道流支付协议

## I. 协议概述与核心原则

### A. 背景：Rooch 网络与微支付需求

Rooch 是一个高性能的模块化区块链网络，旨在为大规模去中心化应用提供基础设施。在 Rooch 生态中，许多应用场景，如 AI 代理间的服务调用、游戏内的道具交易、物联网设备间的数据交换等，都表现为高频、小额的“微支付”形态。

传统的链上交易因其固有的延迟和成本，难以高效支持此类微支付场景。为了解决这一挑战，我们设计了一套专为 Rooch 网络优化的单向状态通道流支付协议。此协议的核心思想是：将绝大多数支付状态的更新转移到链下处理，仅在通道开启、关闭和出现争议时与链上交互，从而在保证资金安全的前提下，最大化支付效率和灵活性。

### B. 核心优势

本方案旨在解决 Rooch 应用在微支付场景下的核心挑战，具备以下优势：

1.  **极致的性能与低成本**：通过将计费和支付确认过程完全置于链下，实现了近乎即时的支付体验，同时将链上交易成本降至最低。
2.  **灵活的计费模型**：将复杂的计费逻辑（如按 API 调用次数、按 Token 消耗量、动态定价等）完全移至链下，使服务提供方可以灵活调整其商业模式，而无需修改链上合约。
3.  **与 Rooch 原生账户集成**：协议直接利用 Rooch 的原生账户体系及其 DID 模型，通过*子通道*授权特定密钥，为多设备、多会话场景提供了优雅且安全的解决方案。
4.  **异步与非合作安全性**：通过引入挑战期和欺诈证明机制，协议有效防止了恶意行为，并降低了双方持续在线监控的负担，同时支持通过“瞭望塔”服务实现委托监控。
5.  **确定性与可发现性**：通道 ID 由支付方和接收方地址确定性生成，无需链上索引即可查询，极大简化了客户端实现。

## II. 技术方案：基于共享资金池与子通道授权的架构

本协议的核心设计思想是将**资金托管**与**通道状态管理**彻底分离，并利用 Rooch 的原生 DID 功能来**预授权**支付密钥，从而实现一个既高效又安全的支付系统。

### A. 核心架构

该架构由三个关键组件构成：

1.  **`PaymentHub` (支付中心)**: 一个归属于支付方账户的**唯一命名对象**。它内部直接包含一个 `Object<MultiCoinStore>`，使其能统一管理**多种不同类型**的代币。这不仅解决了资金碎片化问题，还为协议添加了一个可扩展的策略层。
2.  **`PaymentChannel<CoinType>` (支付通道)**: 一个轻量级的、**泛型**的通道状态对象，拥有**确定性 ObjectID**。它自身不直接持有资金，而是通过 `payment_hub_id` 链接到一个 `PaymentHub`。每个通道都严格绑定一种 `CoinType`，确保了类型安全。
3.  **`SubChannel` (子通道)**: 一个**链上**结构，与支付方的一个特定**验证方法 (Verification Method)** 绑定。它在授权时**固化了该验证方法的公钥和类型**，用于处理来自单个设备或会话的并发支付流。每个子通道都有自己独立的 `nonce` 和累积金额。

**架构优势**:
*   **通用支付中心**: 用户只需管理一个 `PaymentHub`，即可支持所有币种的支付通道，极大简化了用户操作。
*   **协议可扩展性**: 可以在 `PaymentHub` 上添加协议级别的策略和限制（如最大通道数、总流出速率等），而无需修改标准的 `MultiCoinStore`。
*   **类型安全**: `PaymentChannel<CoinType>` 的泛型设计确保了每个通道的账本和链下凭证在编译时就是类型安全的。
*   **安全的多设备支持**: 通过 `authorize_sub_channel` 操作，支付方可以显式授权一个 DID 验证方法。合约会将该方法的公钥信息**固化到链上**。后续所有签名验证都基于这个固化信息，**彻底摆脱了对 DID 文档实时状态的依赖**，防止了因用户轮换或删除 DID 密钥而导致通道资金被锁死的问题。
*   **可发现性**: 通道 ID 可通过 `calc_channel_object_id(sender, receiver)` 预测，客户端可以轻松检查通道是否存在，或重新激活已关闭的通道。

### B. 链上状态定义

```move
// 用于生成确定性 ObjectID 的 Key
struct ChannelKey has copy, drop, store {
    sender: address,
    receiver: address,
}

// 支付中心对象 (非泛型，账户唯一命名对象)
struct PaymentHub has key {
    multi_coin_store: Object<MultiCoinStore>,
    // 未来可扩展的策略字段...
}

// 支付通道对象 (泛型)
struct PaymentChannel<phantom CoinType: store> has key {
    sender: address,
    receiver: address,
    payment_hub_id: ObjectID, // 链接到 PaymentHub 对象
    sub_channels: Table<String, SubChannel>, // Key 是 DID VM 的 fragment
    status: u8, // 0: Active, 1: Cancelling, 2: Closed
    cancellation_info: Option<CancellationInfo>,
}

// 子通道的链上状态记录 (包含授权元数据)
struct SubChannel has store {
    // --- 授权元数据 (一次性设置) ---
    pk_multibase: String,
    method_type: String,
    
    // --- 状态数据 (随操作演进) ---
    last_claimed_amount: u256,
    last_confirmed_nonce: u64,
}

// 用于支付方单方面取消通道时的状态记录
struct CancellationInfo has copy, drop, store {
    initiated_time: u64, // 区块时间戳
    pending_amount: u256,
}
```

### C. 核心操作流程

#### 1. 设置阶段

*   **`create_payment_hub`**: 支付方首次调用此函数，将为其地址创建一个唯一的、持久的 `PaymentHub`。若已存在则无操作。
*   **`deposit_to_hub<CoinType>`**: 支付方调用此函数，将特定类型和数量的 `Coin<CoinType>` 存入其 `PaymentHub` 中。

#### 2. 通道开启与子通道授权 (关键步骤)

*   **`open_channel<CoinType>`**: 支付方为接收方调用此函数。合约会计算出确定性的 `channel_id`。
    *   如果通道**不存在**，则创建一个新的 `PaymentChannel` 对象。
    *   如果通道**已存在且状态为 `Closed`**，则会**重新激活**该通道，并保留所有已授权的子通道。
    *   如果通道**已存在且为 `Active`**，则报错。
*   **`authorize_sub_channel<CoinType>`**: 支付方必须调用此函数来**授权**一个 DID 验证方法 (VM)。
    *   合约会验证该 VM 属于调用者且拥有 `authentication` 权限。
    *   然后将该 VM 的公钥和类型固化到 `SubChannel` 结构中，并存入 `sub_channels` 表。
    *   **这是一个必须的步骤**，在此之后，该子通道才能被用于支付。
*   **`open_channel_with_sub_channel<CoinType>` (便民函数)**: 为了简化操作，协议提供了此函数，可将上述两步合并为一次调用。

#### 3. 链下流式支付 (基于子通道)

*   **`SubRAV` 结构**: 链下凭证 `SubRAV` (Sub-channel Receipts and Vouchers) 是所有链下交互的核心。
    ```rust
    // 用于哈希计算的链上结构
    struct SubRAV {
        channel_id: ObjectID, // PaymentChannel 的 ObjectID
        vm_id_fragment: String, // 支付方 DID 的验证方法 fragment (e.g., "key-1")
        accumulated_amount: u256,
        nonce: u64,
    }
    ```
*   **链下交互**: 支付方的设备使用其对应的私钥（该私钥与 `authorize_sub_channel` 授权的公钥对应）对 `bcs::to_bytes(&sub_rav)` 的哈希值进行签名，并与接收方进行高频的状态更新。不同子通道（不同设备）的支付流互不干扰。

#### 4. 中途提款 (`claim_from_channel`)

这是该架构的核心优势体现。接收方可以在不关闭通道的情况下，随时提取任何一个子通道的累积资金。

*   **链上操作**: 接收方调用 `claim_from_channel<CoinType>`，提交 `channel_id`，以及最新的 `SubRAV` 和签名。
*   **合约逻辑**:
    1.  验证接收方身份，以及通道状态为 `Active`。
    2.  从 `sub_channels` 表中找到对应的 `SubChannel`，使用其中存储的 `pk_multibase` 和 `method_type` 对 `SubRAV` 的签名和 `nonce` 进行验证。
    3.  从 `PaymentChannel` 获取 `payment_hub_id`，并借出 `PaymentHub` 对象。
    4.  计算增量金额：`incremental_amount = accumulated_amount - sub_channel.last_claimed_amount`。
    5.  **从 `PaymentHub` 的 `MultiCoinStore` 中，使用 `multi_coin_store::withdraw` 提取 `incremental_amount` 对应的代币，然后转给接收方。**
    6.  更新 `PaymentChannel` 中该子通道的链上状态 (`last_claimed_amount`, `last_confirmed_nonce`)。

#### 5. 关闭、取消与仲裁

通道的关闭和争议解决流程与基础模型类似，但所有资金操作都将指向支付中心。

*   **合作关闭 (`close_channel`)**: 接收方提交所有子通道的最终 `SubRAV` 证明，合约验证后，从支付中心结算最后一笔款项，并将 `PaymentChannel` 标记为 `Closed`。
*   **单方面取消 (`initiate_cancellation` & `dispute_cancellation` & `finalize_cancellation`)**: 支付方可以单方面发起取消，进入挑战期。接收方可以在挑战期内提交更新的 `SubRAV` 进行争议。挑战期结束后，最终结算的资金同样来自支付中心。

## III. Move 模块接口设计 (概念性)

```move
// file: sources/payment_channel.move
module rooch_framework::payment_channel {
    // ... imports

    // === 核心函数签名 ===

    // --- 设置阶段 ---
    public entry fun create_payment_hub();
    public entry fun deposit_to_hub_entry<CoinType: key + store>(sender: &signer, receiver: address, amount: u256);
    public fun deposit_to_hub<CoinType: key + store>(account_addr: address, coin: Coin<CoinType>);

    // --- 通道管理 ---
    public entry fun open_channel_entry<CoinType: key + store>(sender: &signer, receiver: address);
    public entry fun authorize_sub_channel_entry<CoinType: key + store>(sender: &signer, channel_id: ObjectID, vm_id_fragment: String);

    // --- 便民函数 ---
    public entry fun open_channel_with_sub_channel_entry<CoinType: key + store>(sender: &signer, receiver: address, vm_id_fragment: String);
    public entry fun open_channel_with_multiple_sub_channels_entry<CoinType: key + store>(sender: &signer, receiver: address, vm_id_fragments: vector<String>);

    // --- 支付与结算 ---
    public entry fun claim_from_channel_entry<CoinType: key + store>(
        account: &signer, // Must be receiver
        channel_id: ObjectID,
        sender_vm_id_fragment: String,
        sub_accumulated_amount: u256,
        sub_nonce: u64,
        sender_signature: vector<u8>
    );

    public entry fun close_channel_entry<CoinType: key + store>(
        receiver: &signer,
        channel_id: ObjectID,
        serialized_proofs: vector<u8>, // bcs::to_bytes(&vector<CloseProof>)
    );
    
    // --- 争议处理 ---
    public entry fun initiate_cancellation_entry<CoinType: key + store>(sender: &signer, channel_id: ObjectID);
    public entry fun dispute_cancellation_entry<CoinType: key + store>(
        account: &signer, // Must be receiver
        channel_id: ObjectID,
        sender_vm_id_fragment: String,
        dispute_accumulated_amount: u256,
        dispute_nonce: u64,
        sender_signature: vector<u8>
    );
    public entry fun finalize_cancellation_entry<CoinType: key + store>(channel_id: ObjectID);


    // === 视图函数 ===
    public fun get_payment_hub_id(owner: address): ObjectID;
    public fun get_channel_id<CoinType: store>(sender: address, receiver: address): ObjectID;
    public fun get_channel_info<CoinType: store>(channel_id: ObjectID): (address, address, ObjectID, u8);
    public fun get_sub_channel_state<CoinType: store>(channel_id: ObjectID, vm_id_fragment: String): (u256, u64);
    
    // === 内部逻辑 ===
    fun verify_sender_signature<CoinType: key + store>(
        channel: &PaymentChannel<CoinType>,
        channel_id: ObjectID,
        vm_id_fragment: String,
        accumulated_amount: u256,
        nonce: u64,
        signature: vector<u8>
    ): bool {
        let msg_hash = get_sub_rav_hash(...);
        let sub_channel = table::borrow(&channel.sub_channels, vm_id_fragment);
        
        // 调用 DID 模块的通用验证函数，使用固化在 sub_channel 里的公钥信息
        did::verify_signature_by_type(msg_hash, signature, &sub_channel.pk_multibase, &sub_channel.method_type)
    }
}
```

## IV. 应用场景示例：AI 代理协议（以 Nuwa 为例）

为了更好地理解该支付协议的应用，我们以一个构建在 Rooch 上的 AI 代理协议（例如 Nuwa 协议）为例。在该场景中，一个客户端 AI 代理需要向多个不同的服务（如 LLM 推理、数据存储、图像生成）支付费用。

1.  **一次性设置支付中心**: 客户端代理调用 `create_payment_hub`，并调用 `deposit_to_hub` 存入 1000 RGas 代币作为其所有微支付的总预算。
2.  **开启通道并授权设备**: 
    *   客户端为 LLM 网关调用 `open_channel_with_sub_channel<RGas>(llm_gateway_addr, "my-phone-key")`，一步完成通道开启和手机设备授权。
    *   客户端为数据存储服务调用 `open_channel_with_sub_channel<RGas>(storage_service_addr, "my-laptop-key")`，为笔记本设备授权。
3.  **并发链下支付**: 
    *   客户端代理在手机上调用 LLM API，使用手机的密钥（对应 "my-phone-key"）对 `SubRAV` 签名。
    *   同时，其在笔记本电脑上的脚本自动备份数据，使用笔记本的密钥（对应 "my-laptop-key"）对发往存储服务的 `SubRAV` 进行签名。
4.  **独立提款**: 
    *   LLM 网关在累积了 50 RGas 的费用后，调用 `claim_from_channel` 从支付中心中提款。
    *   数据存储服务在累积了 10 RGas 后，也独立调用 `claim_from_channel` 提款。

这个流程充分展示了该统一方案的灵活性和高资金效率。

## V. 结论与未来展望

本方案为 Rooch 网络设计了一套统一且强大的单向状态通道流支付协议。通过将**支付中心 (PaymentHub)** 与 **DID 子通道授权**无缝结合，该方案不仅解决了基础的微支付需求，还通过**固化公钥**和**确定性ID**的设计，优雅地处理了资金碎片化、多设备并发和状态发现的复杂问题，为开发者提供了强大而灵活的支付基础设施。

这为 Rooch 生态（如 Nuwa 协议）中的应用提供了“一步到位”的支付解决方案，能够支撑从简单到复杂的各类业务场景。

### C. 探索更高级的密码学原语

为了进一步增强协议的能力和应用范围，未来可以探索集成更高级的密码学技术：

*   **条件支付与可锁签名 (Conditional Payments & Lockable Signatures)**: 虽然在当前的双边单向模型中不是必需的，但可锁签名是实现网络化支付（如 A->B->C 多跳支付）和原子互换的关键。通过引入基于哈希时间锁（HTLC）的逻辑，可以将该协议从一个双边工具升级为一个网络化的支付基础设施，极大地扩展其应用场景。

*   **哈希链与支付计量 (Hash Chains & Payment Metering)**: 作为当前 `u64` nonce 的一种替代方案，哈希链可以为按次计费的场景提供更精细的支付计量。支付方可以预先生成一条哈希链，每次支付消费链上的一个哈希，为服务提供一种“即用即付”的精确凭证。虽然这会增加链下管理的复杂性，但为特定应用场景提供了额外的灵活性。

## VI. 子通道停用（整通道重置）方案

> 场景：某台设备或其私钥丢失，Sender 希望立即阻止该设备继续支付；接受“暂停整条通道再重新开启”带来的中断。

### A. 核心思想

1. **通道世代 (channel_epoch)** —— 在 `PaymentChannel` 里维护 `channel_epoch: u64` 字段。  
2. **关闭时 +1** —— 每次 `close_channel` 或 `finalize_cancellation` 结束时执行 `channel.channel_epoch += 1` 并把 `status` 设为 `Closed`。  
3. **重开通道** —— 调 `open_channel`(或 `open_channel_with_sub_channel`) 把 `status` 改回 `Active`；新通道仍使用相同 `channel_id`，但 `channel_epoch` 已是新值。  
4. **RAV 带世代** —— `SubRAV` 新增字段 `channel_epoch`；验签时要求 `sub_rav.channel_epoch == channel.channel_epoch`，否则直接拒绝。  
5. **清空子通道表** —— 关闭通道时直接 `table::destroy(channel.sub_channels)`，重开后需要重新 `authorize_sub_channel` 进行授权。

### B. 数据结构变更 (概念)

```move
struct PaymentChannel has key {
    sender: address,
    receiver: address,
    coin_type: String,
    sub_channels: Table<String, SubChannel>,
    status: u8,        // 0 Active, 1 Cancelling, 2 Closed
    channel_epoch: u64,   // 每次整通道关闭后 +1
    cancellation_info: Option<CancellationInfo>,
}

struct SubChannel has store {
    pk_multibase: String,
    method_type: String,
    last_claimed_amount: u256,
    last_confirmed_nonce: u64,
    // 无 status 字段 —— 一刀切停止后须重新授权
}

struct SubRAV has copy, drop, store {
    channel_id: ObjectID,
    channel_epoch: u64,          // 新增
    vm_id_fragment: String,
    accumulated_amount: u256,
    nonce: u64,
}
```

### C. 操作流程

| 步骤 | 调用者 | 关键动作 |
|------|--------|---------|
| `open_channel` / `open_channel_with_sub_channel` | Sender | 若不存在则创建，`channel_epoch = 0` |
| 正常支付 | 双方 | RAV 必须携带 `channel_epoch=0` |
| **整通道取消** `initiate_cancellation` → `finalize_cancellation` *或* `close_channel` | Sender / Receiver | 结算后 `channel_epoch += 1` ，`status = Closed` ，`table::destroy(sub_channels)` |
| **重新开启** | Sender | `status = Active`，`channel_epoch` 保持新值；需要重新 `authorize_sub_channel` 授权设备 |
| 后续支付 | 双方 | RAV 必须携带 **新的** `channel_epoch` |

### D. 安全与特性

1. **阻断旧私钥** 旧设备签出的 RAV 携带过期 `channel_epoch`，合约直接拒绝，无需遍历或存额外状态。  
2. **重放防护** `nonce` 仍单调递增；`channel_epoch` + `nonce` 双层保护。  
3. **实现简单** 关闭时仅两步：`channel_epoch += 1`；`destroy(sub_channels)`，O(1) 写操作。  
4. **重新授权** Sender 在重开后选择性为仍有效的设备重新 `authorize_sub_channel`，灵活且显式。  
5. **客户端代价** RAV 结构多 8 bytes；签名与 CLI 逻辑需携带 `channel_epoch` 字段。

### E. 何时选择该方案

* 业务接受“所有设备暂时断流再重新授权”的停机窗口；  
* 更在意合约逻辑简单、Gas 成本低，而非单设备不停机；  
* 客户端尚未上线，统一升级 RAV 结构与签名流程没有历史负担。

---

若将来需要“停用单台设备而不中断其它设备”，可以在此方案之上再为 `SubChannel` 引入 `status` 和 `disable_sub_channel_entry`，与 `generation` 机制并存，两者并不冲突。
