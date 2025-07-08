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

## II. 技术方案：基于 Move Object 的单向支付通道

协议的核心是一个部署在 Rooch 链上的 Move 模块，它定义了一个名为 `PaymentChannel` 的核心对象（Object）。这个对象作为支付流的信任锚点和最终结算层，而大部分交易和状态更新则在链下进行。

### A. 参与方与核心状态

*   **支付方 (Sender)**：任何 Rooch 账户，在应用层可以是一个客户端、一个设备或一个 AI 代理。
*   **接收方 (Receiver)**：任何 Rooch 账户，通常是服务提供商。
*   **链上 `PaymentChannel` 对象**：一个共享对象（Shared Object），托管资金并作为争议解决的信任锚点。

**核心链上状态 (`PaymentChannel` Object 的字段):**

```move
struct PaymentChannel<CoinType: store> has key {
    id: ObjectID,
    sender: address,
    receiver: address,
    balance: Balance<CoinType>, // 托管的资金
    last_claimed_amount: u256,
    last_confirmed_nonce: u64,
    status: u8, // 0: Active, 1: Cancelling, 2: Closed
    cancellation_info: Option<CancellationInfo>,
}

struct CancellationInfo has store {
    initiated_time: u64, // 区块时间戳
    pending_amount: u256,
}
```

*   `status`: 通道状态，`Active` (活跃), `Cancelling` (取消中), `Closed` (已关闭)。
*   `cancellation_info`: 当支付方发起取消时，记录取消时间和声称的累积金额。

### B. 核心操作流程

#### 1. 开启支付流 (`open_stream`)

*   **链上操作**：
    *   **初始化**：支付方 (Sender) 调用 Move 模块的 `open_stream` 函数，指定接收方 (Receiver) 的地址，并传入一定数量的代币（`Coin<T>`）作为抵押金。
    *   **对象创建**：函数创建一个新的 `PaymentChannel` 共享对象，并记录所有初始参数。该对象的 ID 即为 `channel_id`。
    *   **与 Rooch 账户集成**：通道在逻辑上绑定到 Sender 和 Receiver 的 Rooch 账户地址。支付方可以使用其账户下的任何一个有效密钥来签署开启通道的交易。

#### 2. 链下更新 (流式支付)

*   **链下操作**：
    *   **服务消费与计费**：当支付方消费接收方的服务时，接收方根据其链下定价策略计算累积费用。
    *   **生成累积凭证 (RAV)**：接收方生成一个“已累积价值收据”（Receipt of Accumulated Value, RAV）。
        ```rust
        // RAV 结构 (链下定义)
        struct RAV {
            channel_id: ObjectID,
            accumulated_amount: u256,
            nonce: u64,
        }
        ```
    *   **双重签名共识**：
        1.  接收方对 RAV 进行签名，并将其发送给支付方。
        2.  支付方验证 RAV 的有效性（包括接收方签名、`nonce` 是否递增、金额计算是否符合预期等）。
        3.  验证通过后，支付方用自己的私钥对该 RAV 进行**签名确认**，并将这个**双重签名**的 RAV 返回给接收方。
        4.  接收方收到并存储这个双重签名的 RAV，作为最新的有效凭证。

#### 3. 关闭/取消 (`close_channel` / `initiate_cancellation`)

*   **链上操作**：
    *   **接收方关闭 (`close_channel`)**：
        *   接收方可以随时调用 `close_channel` 函数，并提交其持有的最新双重签名 RAV。
        *   合约验证 RAV 有效后，立即进行结算，无需挑战期。
    *   **支付方取消 (`initiate_cancellation`)**：
        *   支付方可以随时调用 `initiate_cancellation` 函数，并提交其持有的最新 RAV。
        *   合约记录支付方声称的累积金额，并将通道状态标记为 `Cancelling`，进入一个预设的**挑战期**（例如，1 天）。

#### 4. 结算/仲裁 (`dispute` / `finalize_cancellation`)

*   **链上操作**：
    *   **争议解决 (`dispute`)**：
        *   在挑战期内，如果接收方发现支付方提交的 RAV 是过时的，可以调用 `dispute` 函数，并提交一个拥有更高 `nonce`（或在 `nonce` 相同时金额更高）的有效双重签名 RAV 作为欺诈证明。
        *   智能合约验证欺诈证明。如果有效，合约会更新 `pending_amount` 为接收方提交的更高金额。
    *   **最终结算 (`finalize_cancellation`)**：
        *   挑战期结束后，任何一方都可以调用 `finalize_cancellation` 函数。
        *   合约根据最终确定的 `pending_amount` 进行资金分配：将累积金额转给接收方，剩余抵押金退还给支付方。
        *   通道状态被标记为 `Closed`，相关链上数据被清理或归档。

## III. 应用场景示例：AI 代理协议（以 Nuwa 为例）

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

## IV. Move 模块接口设计 (概念性)

以下是一个概念性的 Move 模块接口，用于说明协议的核心逻辑。

```move
// file: sources/streaming_payment.move
module rooch_examples::streaming_payment {
    use std::option::{Self, Option};
    use std::signer;
    use rooch_framework::object::{Self, Object, ObjectID};
    use rooch_framework::balance::{Self, Balance};
    use rooch_framework::coin::Coin;
    use rooch_framework::crypto::schnorr; // Or other signature schemes
    use rooch_framework::timestamp;
    use rooch_framework::transfer;
    use moveos_std::type_info;
    use moveos_std::bcs;
    use moveos_std::u256;

    // === Constants ===
    const STATUS_ACTIVE: u8 = 0;
    const STATUS_CANCELLING: u8 = 1;
    const STATUS_CLOSED: u8 = 2;
    const CHALLENGE_PERIOD_SECONDS: u64 = 86400; // 1 day

    // === Structs ===
    struct RAV has drop {
        channel_id: ObjectID,
        accumulated_amount: u256,
        nonce: u64,
    }

    struct CancellationInfo has store {
        initiated_time: u64,
        pending_amount: u256,
    }

    struct PaymentChannel<CoinType: store> has key {
        sender: address,
        receiver: address,
        balance: Balance<CoinType>,
        last_claimed_amount: u256,
        last_confirmed_nonce: u64,
        status: u8,
        cancellation_info: Option<CancellationInfo>,
    }

    // === Public Functions ===

    /// Sender opens a payment stream and deposits funds.
    public entry fun open_stream<CoinType: store>(
        sender: &signer,
        receiver: address,
        deposit: Coin<CoinType>
    ) {
        // ... function body ...
    }

    /// Receiver closes the channel with the latest doubly-signed RAV.
    /// Settlement is immediate.
    public entry fun close_channel<CoinType: store>(
        receiver: &signer,
        channel_obj: &mut Object<PaymentChannel<CoinType>>,
        accumulated_amount: u256,
        nonce: u64,
        sender_signature: vector<u8>
    ) {
        // ... function body ...
    }

    /// Sender initiates the cancellation process, starting a challenge period.
    public entry fun initiate_cancellation<CoinType: store>(
        sender: &signer,
        channel_obj: &mut Object<PaymentChannel<CoinType>>,
        final_accumulated_amount: u256,
        final_nonce: u64,
        sender_signature: vector<u8>
    ) {
        // ... function body ...
    }

    /// During the challenge period, the receiver can submit a newer RAV as proof.
    public entry fun dispute<CoinType: store>(
        receiver: &signer,
        channel_obj: &mut Object<PaymentChannel<CoinType>>,
        disputed_accumulated_amount: u256,
        disputed_nonce: u64,
        sender_signature: vector<u8>
    ) {
        // ... function body ...
    }

    /// After the challenge period, anyone can finalize the settlement.
    public entry fun finalize_cancellation<CoinType: store>(
        channel_obj: &mut Object<PaymentChannel<CoinType>>
    ) {
        // ... function body ...
    }

    /// Sender can add more funds to an active channel.
    public entry fun top_up<CoinType: store>(
        sender: &signer,
        channel_obj: &mut Object<PaymentChannel<CoinType>>,
        additional_deposit: Coin<CoinType>
    ) {
        // ... function body ...
    }

    // === Internal Helper Functions ===

    fun get_rav_hash(channel_id: ObjectID, accumulated_amount: u256, nonce: u64): vector<u8> {
        let rav = RAV { channel_id, accumulated_amount, nonce };
        bcs::to_bytes(&rav)
    }

    fun verify_sender_signature(
        channel_id: ObjectID,
        sender_address: address,
        accumulated_amount: u256,
        nonce: u64,
        signature: vector<u8>
    ): bool {
        let msg_hash = get_rav_hash(channel_id, accumulated_amount, nonce);
        // The public key can be retrieved from the sender's account storage
        // let public_key = rooch_framework::account::get_public_key(sender_address);
        // schnorr::verify(&public_key, &msg_hash, &signature)
        true // Placeholder
    }
}
```

## V. 高级方案：集成 DID 的“主-子通道”架构

为了从根本上解决多设备并发更新的状态同步难题，并与 Rooch 的 DID 身份层深度集成，我们引入一个更高级的“主-子通道”（Hub-and-Spoke）架构。

### A. 核心思想

我们不再让所有设备共同竞争更新一个单一的状态（nonce），而是为每个设备或会话（session）创建一个逻辑上的**子通道 (Sub-Channel)**。所有子通道都链接到一个唯一的**主通道 (Master Channel)**。

*   **主通道**：链上的 `PaymentChannel` 对象，负责托管总资金，并作为最终结算的信任根。
*   **子通道**：完全在链下维护的逻辑通道，每个子通道对应一个设备/会话。它有自己独立的 `sub_nonce` 和累积金额 `sub_accumulated_amount`。

支付流程变为：设备在各自的子通道上与接收方进行高频的状态更新。接收方可以定期或在需要时，将一个或多个子通道的最新状态“合并”到主通道上进行链上结算。

### B. 与 `rooch_framework::did` 的集成

此方案的核心在于将子通道的身份验证与 Rooch 的 DID 模块绑定。子通道不再由裸公钥标识，而是由支付方 DID 文档中的一个**验证方法 (Verification Method)** 来标识和授权。

**扩展后的链上状态:**

```move
struct SubChannelState has store {
    last_claimed_amount: u256,
    last_confirmed_nonce: u64,
}

struct PaymentChannel<CoinType: store> has key {
    sender: address, // 支付方的 Rooch 账户地址
    receiver: address,
    balance: Balance<CoinType>,
    // key 是子通道的唯一标识符 (如 verification_method_id 的哈希)
    sub_channels: Table<vector<u8>, SubChannelState>,
    status: u8,
    cancellation_info: Option<CancellationInfo>,
}
```

### C. 扩展后的核心操作

#### 1. 链下更新 (基于子通道与 Verification Method)

*   **RAV 结构扩展**：链下凭证 RAV 需要包含子通道及验证方法信息。
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

#### 2. 结算 (`claim_from_sub_channel`)

*   **链上操作**：
    *   接收方调用新的 `claim_from_sub_channel` 函数，并提交目标子通道的最新 `SubRAV` 及其签名。
    *   **合约逻辑**：
        1.  从 `SubRAV` 中解析出 `verification_method_id`。
        2.  调用 `verify_sender_signature` 核心函数，该函数会利用 `did` 模块进行验证。
        3.  验证通过后，计算增量金额并结算。

### D. Move 模块接口与验证逻辑调整

```move
// file: sources/streaming_payment.move
module rooch_examples::streaming_payment {
    // ... (use 语句需要增加 rooch_framework::did)
    use rooch_framework::did;
    use moveos_std::string::String;
    // ...

    // ... (结构体定义如上文所示)

    /// 接收方从一个特定的子通道提款
    public entry fun claim_from_sub_channel<CoinType: store>(
        receiver: &signer,
        channel_obj: &mut Object<PaymentChannel<CoinType>>,
        // 不再传递公钥，而是传递 Verification Method ID 的 fragment 部分
        sender_vm_id_fragment: String,
        sub_accumulated_amount: u256,
        sub_nonce: u64,
        sender_signature: vector<u8>
    ) {
        let channel = object::borrow_mut(channel_obj);
        assert!(signer::address_of(receiver) == channel.receiver, ENotReceiver);
        assert!(channel.status == STATUS_ACTIVE, EChannelNotActive);

        // 1. 验证签名
        assert!(
            verify_sender_signature(
                object::id(channel_obj),
                channel.sender,
                &sender_vm_id_fragment,
                sub_accumulated_amount,
                sub_nonce,
                &sender_signature
            ),
            EInvalidSenderSignature
        );

        // 2. 获取或创建子通道的链上状态
        let sub_channel_id = bcs::to_bytes(&sender_vm_id_fragment);
        let sub_channel_state = if (table::contains(&channel.sub_channels, sub_channel_id)) {
            table::borrow_mut(&mut channel.sub_channels, sub_channel_id)
        } else {
            table::add(&mut channel.sub_channels, sub_channel_id, SubChannelState {
                last_claimed_amount: 0,
                last_confirmed_nonce: 0,
            });
            table::borrow_mut(&mut channel.sub_channels, sub_channel_id)
        };

        // 3. 验证 nonce 并结算
        // ...
    }

    // === Internal Helper Functions ===

    fun get_sub_rav_hash(
        master_channel_id: ObjectID,
        vm_id_fragment: &String,
        accumulated_amount: u256,
        nonce: u64
    ): vector<u8> {
        // ... (哈希内容应包含所有 SubRAV 字段)
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

## VI. 结论与未来展望

本方案为 Rooch 网络设计了一个原生的单向状态通道流支付协议，旨在高效、安全地支持其生态应用间的微支付。通过引入与 DID 标准深度集成的“主-子通道”架构，该方案不仅解决了基础的微支付需求，还优雅地处理了 Rooch 账户在多设备环境下的状态同步与权限管理难题，为开发者提供了强大而灵活的支付基础设施。

像 Nuwa 这样的高级 AI 代理协议，可以利用此支付通道来实现其内部经济激励和资源调度。未来，Rooch 可以进一步探索以下高级概念，以增强协议的能力：
*   **瞭望塔 (Watchtowers)**：标准化瞭望塔服务，接收方可以将最新的 RAV 委托给瞭望塔，由其自动监控链上活动并在发生争议时代表接收方提交欺诈证明。
*   **零知识证明 (ZKPs)**：研究将 ZKPs 集成到欺诈证明机制中，以进一步缩短挑战期，实现更快的最终性，并增强支付细节的隐私性。
*   **广义状态通道**：将支付通道扩展为更广义的状态通道，允许应用在链下执行更复杂的协作和状态更新，而不仅仅是支付。
