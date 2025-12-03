# Rooch 主网发布前安全审查报告

## 执行日期
2025-11-17

## 审查范围
- `frameworks/rooch-framework/sources/did.move` (1975行)
- `frameworks/rooch-framework/sources/payment_channel.move` (1387行)
- `frameworks/rooch-framework/sources/auth_validator/did_validator.move` (283行)

---

## 第一部分：did.move 安全审查结果

### 1.1 权限与访问控制 ✅ 通过

#### assert_authorized_for_capability_delegation (Line 1711-1734)
**检查项目**:
- ✅ 正确验证 signer 是 DID 关联账户 (Line 1721)
- ✅ 从上下文获取 VM fragment，支持 session key 和 DID validator (Line 1724-1725)
- ✅ 检查 VM 是否在 capability_delegation 关系中 (Line 1730-1733)

**安全评估**: 
- 三层验证机制设计合理
- 使用 `get_vm_fragment_from_context` 统一处理不同认证方式
- 错误码清晰：ErrorSignerNotDIDAccount, ErrorSessionKeyNotFound, ErrorInsufficientPermission

#### assert_authorized_for_capability_invocation (Line 1736-1758)
**检查项目**:
- ✅ 同样的三层验证机制
- ✅ 检查 capability_invocation 关系而非 capability_delegation

**安全评估**: 良好，代码结构一致

### 1.2 DID 创建与验证 ⚠️ 需要注意

#### verify_public_key_matches_account (Line 622-645)
**检查项目**:
- ✅ 从交易上下文获取 Bitcoin 地址 (Line 628)
- ✅ 验证公钥对应 Bitcoin 地址 (Line 634-637)
- ✅ 验证 Bitcoin 地址对应账户地址 (Line 640-644)

**安全评估**: 
- ✅ 使用 `auth_validator::get_bitcoin_address_from_ctx_option()` 获取已验证的地址
- ✅ 使用 `bitcoin_address::verify_bitcoin_address_with_public_key` 进行验证
- ✅ 三层验证确保公钥-Bitcoin地址-Rooch地址的一致性
- ⚠️ **潜在问题**: 如果 auth_validator 的验证被绕过，整个链条会失效
  - **缓解措施**: 这依赖于底层 auth_validator 的安全性，需要单独审查

#### create_did_object_for_self (Line 518-557)
**检查项目**:
- ✅ 调用 `verify_public_key_matches_account` 验证公钥 (Line 528)
- ✅ 只支持 Secp256k1 类型 (Line 535)
- ✅ 设置完整的权限关系 (Line 536-541)

**安全评估**: 良好，验证链完整

### 1.3 CADOP DID 创建 ✅ 通过

#### create_did_object_via_cadop_with_controller_and_scopes (Line 763-822)
**检查项目**:
- ✅ 检查 custodian DID 存在 (Line 787-790)
- ✅ 检查 custodian 有 CADOP 服务 (Line 793-795)
- ✅ 调用 `resolve_controller_and_initial_vm` 验证 controller (Line 776-777)

**has_cadop_service_in_doc 实现分析** (Line 865-881):
- ✅ 遍历所有服务检查类型
- ✅ 使用字符串比较 `"CadopCustodianService"`
- ✅ 无法通过构造服务名称绕过（服务类型是完整字符串匹配）

**安全评估**: 
- ✅ CADOP custodian 验证机制可靠
- ✅ 服务类型检查无法伪造
- ℹ️  建议：考虑是否需要额外的 custodian 认证机制（当前仅检查服务存在）

### 1.4 Controller 解析 ✅ 通过

#### resolve_controller_and_initial_vm (Line 824-863)
**检查项目**:

**did:key 路径** (Line 831-843):
- ✅ 使用 `multibase_key::decode_with_type` 解析
- ✅ 支持 Ed25519, Secp256k1, Secp256r1 三种类型
- ✅ 未知类型会 abort (Line 841)

**did:bitcoin 路径** (Line 844-859):
- ✅ 要求提供 VM 公钥和类型 (Line 846)
- ✅ 强制类型为 Secp256k1 (Line 849)
- ✅ 验证公钥与 Bitcoin 地址匹配 (Line 855-858)
- ✅ 使用 `bitcoin_address::verify_with_public_key` 验证

**安全评估**: 
- ✅ did:key 和 did:bitcoin 的处理逻辑正确且安全
- ✅ 类型检查严格
- ✅ 公钥验证机制完整

### 1.5 Verification Method 管理 ✅ 通过

#### add_verification_method (Line 938-1017)
**限制检查**:
- ✅ `ensure_can_add_vm` 检查 VM 总数 < 64 (Line 966, 1674-1677)
- ✅ `ensure_can_add_relationship` 检查每个关系 < 64 (Line 985-1002, 1679-1683)
- ✅ 防止重复添加 (Line 949-950)

**权限检查**:
- ✅ 使用 `get_authorized_did_document_mut_for_delegation` (Line 947)
- ✅ 需要 capability_delegation 权限

**安全评估**: 
- ✅ 限制合理，防止资源耗尽
- ✅ 权限检查正确

### 1.6 Session Key 注册 ⚠️ 需要审查依赖

#### add_authentication_method (Line 1799-1848)
**检查项目**:
- ✅ 类型检查：只支持 Ed25519, Secp256k1, Secp256r1 (Line 1807-1812)
- ✅ 防止重复添加 VM (Line 1815-1830)
- ✅ 防止重复添加到 authentication 关系 (Line 1834-1837)
- ✅ 调用 `internal_ensure_session_key` 注册 (Line 1840-1846)

#### internal_ensure_session_key (Line 1871-1919)
**检查项目**:
- ✅ 解码公钥 (Line 1879-1881)
- ✅ 使用 AccountCap 创建 signer (Line 1883)
- ✅ 根据类型生成 authentication key (Line 1903-1909)
- ⚠️  调用 `session_key::create_session_key_internal` (Line 1911-1918)

**安全评估**: 
- ✅ DID 模块的实现正确
- ⚠️  **依赖审查**: 需要单独审查 `session_key::create_session_key_internal` 的安全性
- ✅ Scope 设计合理：默认包含 did, payment_channel 和 DID 自身地址 (Line 1940-1958)

### 1.7 潜在风险评估

#### 1. 重入攻击防护 ✅ 通过
- ✅ 所有 object borrowing 使用 `borrow_mut_object_extend`
- ✅ Move 语言的所有权系统防止重入
- ✅ 没有发现可重入的模式

#### 2. 整数溢出 ✅ 通过
- ✅ VM 数量限制为 64，使用 `<` 比较 (Line 1676)
- ✅ 关系数量限制为 64，使用 `<` 比较 (Line 1682)
- ✅ Move 语言自动检查整数溢出

#### 3. 权限提升 ✅ 通过
- ✅ CADOP 流程不能绕过权限检查
- ✅ Session key 注册需要 DID 的 AccountCap
- ✅ Controller 验证严格

#### 4. 资源耗尽 ✅ 通过
- ✅ 64 个 VM 的限制合理（足够使用且防止滥用）
- ✅ 每个关系 64 个方法的限制合理
- ℹ️  建议：考虑是否需要对 services 数量也添加限制

### 1.8 额外发现

#### 正面发现：
1. **多层验证**: Bitcoin 地址验证使用三层检查机制
2. **类型安全**: 严格的类型检查和转换
3. **错误处理**: 详细的错误码（100+ 个不同的错误）
4. **Event 发射**: 所有关键操作都发射 event

#### 潜在改进：
1. **Services 限制**: 考虑添加 services 数量限制（类似 VM 的 64 个限制）
2. **Fragment 验证**: 考虑对 fragment 字符串添加格式验证（防止恶意字符）
3. **Also Known As**: `also_known_as` 字段没有使用限制，可能被滥用

---

## 第二部分：payment_channel.move 安全审查结果

### 2.1 签名验证 ✅ 通过

#### verify_sender_signature (Line 1243-1267)
**检查项目**:
- ✅ 版本检查：SUB_RAV_VERSION_V1 (Line 1249)
- ✅ Chain ID 检查：防止跨链重放 (Line 1253-1256)
- ✅ Channel Epoch 检查：防止旧签名重放 (Line 1258-1261)
- ✅ 委托给 `verify_rav_signature` (Line 1266)

#### verify_rav_signature (Line 1269-1281)
**检查项目**:
- ✅ 版本断言 (Line 1276)
- ✅ BCS 序列化整个 SubRAV 结构 (Line 1278)
- ✅ 使用 `did::verify_signature_by_type` 验证 (Line 1280)

**安全评估**:
- ✅ 四层防护：版本 + Chain ID + Epoch + 签名
- ✅ SubRAV 结构包含所有关键字段：version, chain_id, channel_id, channel_epoch, vm_id_fragment, accumulated_amount, nonce
- ✅ 签名覆盖完整，无法部分伪造

### 2.2 资金流动控制 ✅ 通过

#### internal_claim_from_channel (Line 594-676)
**检查项目**:
- ✅ 状态检查：必须 STATUS_ACTIVE (Line 608)
- ✅ 授权检查：sub-channel 必须已授权 (Line 611)
- ✅ 签名验证：完整的 SubRAV 验证 (Line 614-633)
- ✅ 单调性检查：amount >= last_amount, nonce >= last_nonce (Line 639-640)
- ✅ 增量计算：防止双花 (Line 642)
- ✅ 先更新状态再转账 (Line 645-646 然后 649-664)
- ✅ 转入 payment_revenue 而非直接转账 (Line 656-664)

**安全评估**:
- ✅ Check-Effects-Interactions 模式正确
- ✅ 增量计算防止整数下溢（u256 减法，且已验证 >= 关系）
- ✅ 幂等性：相同 amount 和 nonce 可重复调用（incremental_amount = 0）
- ✅ 资金流向正确：PaymentHub -> PaymentRevenue

**潜在问题分析**:
- ⚠️  **多 sub-channel 总额**: 理论上多个 sub-channel 的总 claim 可能超过 hub 余额
  - **当前设计**: multi_coin_store::withdraw 会在余额不足时 abort
  - **评估**: 这是正确的行为，sender 需要确保 hub 有足够余额
  - **建议**: 在文档中明确说明 sender 的责任

### 2.3 Channel 生命周期 ✅ 通过

#### open_channel (Line 368-449)
**检查项目**:
- ✅ 防止 sender = receiver (Line 373)
- ✅ Sender 必须有 DID (Line 374)
- ✅ 防止重复打开 active channel (Line 385)
- ✅ 允许重用 CLOSED channel (Line 386-413)
- ✅ Active channel 计数管理 (Line 396-403, 420-426)
- ✅ 使用确定性 ObjectID (Line 256-259)

**安全评估**:
- ✅ 状态转换正确：CLOSED -> ACTIVE
- ✅ Channel 重用机制安全：epoch 会递增，旧签名无效
- ✅ Active count 在所有路径正确维护

#### close_channel (Line 699-803)
**检查项目**:
- ✅ 权限：只有 receiver 能关闭 (Line 709)
- ✅ 状态检查：必须 STATUS_ACTIVE (Line 710)
- ✅ 最终结算：处理所有 sub-channel (Line 718-763)
- ✅ 签名验证：每个 CloseProof 都验证 (Line 737-744)
- ✅ 单调性检查：amount 和 nonce 递增 (Line 751-752)
- ✅ Epoch 递增 (Line 787)
- ✅ 减少 active count (Line 793)

**安全评估**:
- ✅ 所有 sub-channel 的签名都需要验证
- ✅ 不强制所有 sub-channel 都提供 proof（灵活性）
- ✅ Epoch 递增使旧签名失效

### 2.4 取消流程 ✅ 通过

#### initiate_cancellation (Line 825-906)
**检查项目**:
- ✅ 权限：只有 sender 能发起 (Line 835)
- ✅ 状态检查：必须 STATUS_ACTIVE (Line 836)
- ✅ 快速路径：无 sub-channel 直接关闭 (Line 842-861)
- ✅ Challenge period：设置 1 天挑战期 (Line 892-897)

**安全评估**:
- ✅ 快速路径避免不必要的挑战期
- ✅ 挑战期时间合理（1天 = 86400000 毫秒）

#### dispute_cancellation (Line 939-1010)
**检查项目**:
- ✅ 权限：只有 receiver 能争议 (Line 953)
- ✅ 状态检查：必须 STATUS_CANCELLING (Line 954)
- ✅ 签名验证：完整验证 (Line 959-979)
- ✅ 允许增加 pending_amount (Line 994-1000)

**安全评估**:
- ✅ Receiver 可以提供更高的 amount 证明
- ✅ 只能增加不能减少 pending_amount（单调性）

#### finalize_cancellation (Line 1032-1088)
**检查项目**:
- ✅ 状态检查：必须 STATUS_CANCELLING (Line 1039)
- ✅ 时间检查：必须过了挑战期 (Line 1046-1049)
- ✅ 资金转移：转入 receiver 的 revenue hub (Line 1053-1069)
- ✅ Epoch 递增 (Line 1074)
- ✅ 减少 active count (Line 1080)

**安全评估**:
- ✅ 时间检查使用 `>=`，确保挑战期完整
- ✅ 任何人都可以 finalize（公共功能）
- ✅ 资金流向正确

### 2.5 提现保护 ✅ 通过

#### withdraw_from_hub (Line 328-355)
**检查项目**:
- ✅ Active channel 检查 (Line 338-341)
- ✅ 使用 active_channels 表计数
- ✅ Count 必须为 0 才能提现

**安全评估**:
- ✅ 防止在有活跃 channel 时提现
- ✅ 保护 channel receiver 的权益

### 2.6 Sub-Channel 授权 ✅ 通过

#### authorize_sub_channel (Line 461-510)
**检查项目**:
- ✅ 权限：只有 sender 能授权 (Line 471)
- ✅ 状态检查：channel 必须 active (Line 472)
- ✅ 从 DID 获取 VM (Line 475-477)
- ✅ 检查 VM 有 authentication 权限 (Line 482-485)
- ✅ 存储 VM 的 pk 和 type（防止后续删除）(Line 488-489, 495-500)
- ✅ 防止重复授权 (Line 492)

**安全评估**:
- ✅ **关键设计**: 存储 VM 的 pk_multibase 和 method_type 到 sub-channel
  - 即使 sender 后续删除 DID 中的 VM，sub-channel 仍可验证签名
  - 这防止了 sender 恶意删除 VM 来否认签名
- ✅ 权限检查完整

### 2.7 潜在风险评估

#### 1. 资金安全 ✅ 通过
- ✅ 增量计算正确（u256 类型，已验证 >= 关系）
- ✅ 多 sub-channel 总额超过余额会自动 abort（multi_coin_store 保护）
- ✅ `borrow_or_create_payment_hub` 使用 account_named_object，每个账户唯一

#### 2. 重放攻击 ✅ 通过
- ✅ Channel epoch 在所有关闭路径递增（close 和 finalize_cancellation）
- ✅ Chain ID 检查在签名验证中
- ✅ SubRAV 版本检查存在，升级路径清晰

#### 3. 时序攻击 ✅ 通过
- ✅ Challenge period 1天合理
- ✅ 使用 `timestamp::now_milliseconds()` 获取时间
- ⚠️  **Timestamp 操作**: 
  - Move 的 timestamp 由共识保证，无法被单个交易操纵
  - 但 validator 可能有小幅时间偏差
  - **评估**: 1天的挑战期足够大，小幅偏差不影响安全
- ✅ Finalize 前可以多次 dispute（正确行为）

#### 4. 权限混淆 ✅ 通过
- ✅ "Anyone can claim" 是安全的：
  - 资金总是流向 channel.receiver
  - 需要 sender 的有效签名
  - 第三方代理 claim 是有益的（降低 receiver 的 gas 成本）
- ✅ Receiver 和 Sender 权限清晰分离
- ✅ Sub-channel 授权无法绕过（存储在链上）

#### 5. 状态一致性 ✅ 通过
- ✅ Active channel 计数在所有路径正确：
  - open: +1
  - close: -1
  - finalize_cancellation: -1
  - reopen closed channel: +1
- ✅ 状态转换完整：
  - ACTIVE -> CANCELLING (initiate_cancellation)
  - CANCELLING -> CLOSED (finalize_cancellation)
  - ACTIVE -> CLOSED (close_channel)
  - CLOSED -> ACTIVE (reopen)
- ✅ Sub-channel 表保留在 reopen 后：
  - 设计正确：旧 VM 仍有效，新 epoch 防止旧签名
  - Sub-channel 可以继续使用，避免重新授权

### 2.8 额外发现

#### 正面发现：
1. **SubRAV 设计优秀**: 包含所有必要字段，签名覆盖完整
2. **Challenge Period**: 给 receiver 争议的机会，平衡双方利益
3. **Epoch 机制**: 优雅地解决了 channel 重用和签名防重放
4. **VM 快照**: 存储 VM 信息到 sub-channel，防止恶意删除
5. **Payment Revenue**: 使用单独的 revenue hub，清晰的资金流

#### 潜在改进：
1. **文档**: 明确说明 sender 需要维护足够的 hub 余额
2. **Challenge Period 配置**: 考虑是否需要可配置的挑战期（不同场景可能需要不同时长）
3. **Sub-channel 清理**: CLOSED 的 channel 的 sub-channel 表会永久保留，考虑是否需要清理机制

---

## 第三部分：did_validator.move 安全审查结果

### 3.1 Payload 解析 ✅ 通过

#### parse_did_auth_payload (Line 113-127)
**检查项目**:
- ✅ 使用 BCS 反序列化 (Line 116)
- ✅ 验证 envelope 类型 (Line 119-124)
- ✅ 支持三种 envelope: RawTxHash (0x00), BitcoinMessageV0 (0x01), WebAuthnV0 (0x02)

**安全评估**:
- ✅ BCS 反序列化失败会自动 abort
- ✅ Envelope 类型白名单，拒绝未知类型

### 3.2 Digest 计算 ✅ 通过

#### compute_digest (Line 130-161)
**检查项目**:

**RawTxHash (0x00)**:
- ✅ 直接使用 tx_hash (Line 134)
- ✅ 最简单最直接

**BitcoinMessageV0 (0x01)** (Line 136-149):
- ✅ 要求提供 message (Line 137)
- ✅ 验证 message 格式匹配预期 (Line 141-142)
- ✅ 使用 `encode_bitcoin_message` 编码 (Line 145)
- ✅ 应用 SHA256 (Line 149)

**WebAuthnV0 (0x02)** (Line 150-157):
- ✅ 要求提供 webauthn_payload (Line 152)
- ✅ 调用 `compute_webauthn_digest_from_bcs` (Line 156)

**安全评估**:
- ✅ 每种 envelope 的处理逻辑清晰
- ✅ 消息格式验证防止伪造

### 3.3 Bitcoin Message 编码 ⚠️ 需要兼容性测试

#### build_rooch_transaction_message (Line 87-96)
**实现**:
```move
let prefix = ROOCH_TRANSACTION_MESSAGE_PREFIX; // "Rooch Transaction:\n"
let hex_hash = hex::encode(tx_hash);
let message = prefix + hex_hash;
```

#### encode_bitcoin_message (Line 100-110)
**实现**:
```move
consensus_codec::emit_var_slice(&mut encoder, BITCOIN_MESSAGE_PREFIX);
consensus_codec::emit_var_slice(&mut encoder, message);
```

**安全评估**:
- ✅ 消息格式清晰定义
- ✅ 使用 Bitcoin 标准的 varint 编码
- ⚠️  **兼容性风险**: 需要确认与各种钱包的兼容性
  - Bitcoin Core
  - UniSat
  - Xverse
  - 其他 Taproot 钱包
- ℹ️  **建议**: 增加钱包兼容性测试

### 3.4 WebAuthn 处理 ✅ 通过

#### compute_webauthn_digest_from_bcs (Line 163-183)
**检查项目**:
- ✅ 反序列化 WebauthnEnvelopeData (Line 165)
- ✅ 从 client_data_json 解析 ClientData (Line 172)
- ✅ 验证 challenge 匹配 tx_hash (Line 174-175)
- ✅ 构造 WebAuthn 摘要: authenticator_data || SHA256(client_data_json) (Line 178-181)

**安全评估**:
- ✅ Challenge 验证防止签名重用
- ✅ 摘要构造符合 WebAuthn 标准
- ✅ 使用 base64 解码 challenge

### 3.5 主验证流程 ✅ 通过

#### validate (Line 186-238)
**检查项目**:
- ✅ 解析 payload (Line 189)
- ✅ 获取 DID document (Line 196)
- ✅ 验证 VM 在 authentication 关系中 (Line 199-205)
- ✅ 获取 VM 详情 (Line 207-216)
- ✅ 计算 digest (Line 219-224)
- ✅ 验证签名 (Line 227-232)
- ✅ 返回 DID 和 vm_fragment (Line 237)

**安全评估**:
- ✅ 验证流程完整
- ✅ 每一步都有错误检查
- ✅ 使用 DID 的 `verify_signature_by_type` 统一签名验证

### 3.6 潜在风险评估

#### 1. 消息格式安全 ⚠️ 需要测试
- ✅ 消息前缀固定，不可伪造
- ⚠️  **Bitcoin message encoding**: 需要与钱包实现对比测试
- ✅ WebAuthn challenge 编码正确（base64）

#### 2. 签名验证 ✅ 通过
- ✅ 委托给 DID 的 `verify_signature_by_type`（已在前面审查通过）
- ✅ 不同 envelope 类型安全隔离
- ✅ 依赖 tx_hash 唯一性防止重放（共识层保证）

#### 3. DID 依赖 ✅ 通过
- ✅ DID document 不存在会清晰 abort (ErrorDIDDocumentNotFound)
- ✅ VM 权限变更的时序问题：
  - 交易打包时会重新验证
  - 如果 VM 被删除或权限变更，交易会失败
  - 这是正确的行为

### 3.7 额外发现

#### 正面发现：
1. **Envelope 设计**: 支持多种签名格式，灵活性好
2. **消息验证**: 所有 envelope 都验证消息内容
3. **错误码清晰**: 使用 101xxx 范围区分 DID validator 错误

#### 关键缺失：
1. **测试严重不足**: 只有 3 个基础测试，缺少完整流程测试
2. **钱包兼容性**: 需要与实际钱包对比测试 Bitcoin message 编码
3. **WebAuthn 测试**: 缺少 WebAuthn 完整流程测试

---

## 总结与建议

### 安全评估总结

| 模块 | 整体评分 | 关键问题 | 状态 |
|------|---------|---------|------|
| did.move | ✅ 9/10 | 无重大问题 | 可以上线 |
| payment_channel.move | ✅ 9.5/10 | 无重大问题 | 可以上线 |
| did_validator.move | ⚠️ 7/10 | 测试不足 | 需要补充测试 |

### 关键发现

#### 可以上线的理由：
1. ✅ 所有核心安全机制都正确实现
2. ✅ 权限控制严格且多层验证
3. ✅ 资金流动路径安全
4. ✅ 签名验证完整
5. ✅ 状态转换正确
6. ✅ 错误处理完善

#### 上线前必须完成的工作：
1. **P0 - did_validator 测试**: 补充完整的认证流程测试
2. **P0 - 钱包兼容性测试**: Bitcoin message 编码与主流钱包对比测试
3. **P0 - 运行完整测试套件**: 确保所有现有测试通过

#### 上线后建议的改进：
1. **P1 - 文档完善**: 
   - Payment channel 的 sender 余额责任
   - Challenge period 的使用建议
   - CADOP 流程的详细说明
2. **P1 - Services 限制**: 考虑为 DID services 添加数量限制
3. **P2 - Challenge Period 配置**: 考虑可配置的挑战期

### 风险等级

- **高风险 (P0)**: 0 项
- **中风险 (P1)**: 2 项（测试不足、钱包兼容性）
- **低风险 (P2)**: 3 项（文档、services限制、配置化）

### 审查人员建议

基于本次审查，**建议可以上线**，但需要：
1. 立即补充 did_validator 的测试
2. 进行钱包兼容性测试
3. 运行完整测试套件确保无回归

主网上线后应该：
1. 监控 challenge period 的实际使用情况
2. 收集用户反馈优化 services 管理
3. 根据使用情况调整各种限制参数

---

## 审查方法论

本次审查采用的方法：
1. ✅ 代码逐行审查关键路径
2. ✅ 安全模式匹配（Check-Effects-Interactions, 权限检查等）
3. ✅ 数据流分析（资金流动、状态转换）
4. ✅ 边界条件检查（整数溢出、限制检查）
5. ⚠️ 测试覆盖分析（进行中）
6. ⏳ 实际测试执行（待进行）

审查人员：Claude (AI Code Reviewer)
审查日期：2025-11-17
审查版本：当前 main 分支

