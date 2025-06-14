# WebAuthn 签名验证器设计文档

> 适用版本：Rooch Framework

---

## 1. 背景

现代浏览器原生支持的 WebAuthn (Web Authentication) 可以利用平台安全硬件（Touch ID、Windows Hello、Android Biometric 等）对任意数据进行签名验证。

Rooch 希望让用户直接使用 WebAuthn 设备对 **链上交易** 进行签名，并在智能合约里完成验证。由于 WebAuthn 的签名消息格式与传统"直接签哈希"不同，需要一套专门的验证流程（Validator）。

本文档给出链下/链上的完整方案，供实现参考。

---

## 2. WebAuthn 签名数据回顾

认证器返回的数据包含：

| 字段 | 含义 |
| --- | --- |
| `authenticatorData` | 认证器生成的数据块，格式见 WebAuthn 规范（32 字节 `rpIdHash` + 1 字节 `flags` + 4 字节 `signCount` + …） |
| `clientDataJSON` | 浏览器生成的 JSON 字符串（base64url 编码）<br>最重要字段：`type`、`challenge`、`origin` |
| `signature` | 认证器用私钥签名的二进制（P-256 = 64 字节 `r‖s`，Ed25519 = 64 字节） |
| `publicKey` | 注册阶段得到的公钥（压缩格式 33 字节 / Ed25519 32 字节） |

认证器实际签名的数据为：

```
message_webauthn = authenticatorData || SHA-256(clientDataJSON)
```

因此链上验证时 **必须重建上述 message**，不能只验 `tx_hash`。

---

## 3. 链下流程

1. 计算待发送交易的哈希 `tx_hash`（32 字节）。
2. 调用 `startAuthentication()` / `navigator.credentials.get()` 时，将 `challenge` 设置为 `base64url(tx_hash)`。
3. 获取浏览器返回的 `assertion`，收集以下字段：
   - `signature` (raw 64B)
   - `publicKey` （注册时保存）
   - `authenticatorData`
   - `clientDataJSON`
4. 按约定格式打包为 **AuthenticatorPayload**，随交易一起提交到链上。

### 3.1 AuthenticatorPayload 建议格式
```
┌────────┬────────────┬───────────┬───────────────┬───────────────┐
│ 1 B    │ 64 B       │ 33 B      │ var          │ var           │
│ scheme │ signature  │ publicKey │ authenticatorData │ clientDataJSON │
└────────┴────────────┴───────────┴───────────────┴───────────────┘
```
- **scheme**：签名方案编号，示例 `0x04` 表示 `WebAuthn_P256`。
- 其余字段长度不固定时可使用 TLV/BCS 长度前缀编码，也可以直接顺序拼接+链上解析。

---

## 4. 链上验证器实现思路

### 4.1 新增方案编号
```move
const signature_scheme_webauthn_r1: u8 = 0x04; // 与链下保持一致
```

### 4.2 验证流程伪代码
```move
fun validate_webauthn(auth_payload: &vector<u8>, expected_tx_hash: &vector<u8>): vector<u8> {
    // 1. 解析 payload
    let (sig, pubkey, ad, cdj) = decode(auth_payload);

    // 2. 可选安全检查
    assert!(extract_rp_id_hash(&ad) == SHA256(origin_domain), ERR_RP_MISMATCH);

    // 3. 计算 clientDataHash
    let cd_hash = sha256(&cdj);

    // 4. 组成 WebAuthn message
    let msg = vector::concat(&ad, &cd_hash);

    // 5. 调用 ecdsa_r1 验证
    assert!(ecdsa_r1::verify(&sig, &pubkey, &msg), ERR_SIG_INVALID);

    // 6. 解析 clientDataJSON 并比对 challenge（可选）
    // 确保 challenge == base64url(expected_tx_hash)

    // 7. 更新 signCount （重放保护，可选）

    // 8. 返回 auth_key 供后续逻辑使用
    session_key::secp256r1_public_key_to_authentication_key(&pubkey)
}
```

### 4.3 与现有 `session_validator` 的集成方式

- **方案 A**：在 `session_validator` 增加新的 `scheme` 分支，重用存量 Session 逻辑。
- **方案 B**：编写独立模块 `webauthn_validator.move`，由 `transaction_validator` 根据 scheme 路由调用。

推荐 **方案 B**，避免核心验证器臃肿。

---

## 5. 安全考虑

| 风险 | 对策 |
| --- | --- |
| 重放攻击 | 使用 `signCount` + 全局表保存最新计数，拒绝旧值 |
| 钓鱼站伪造 | 在链上比对 `rpIdHash` 与固定的域名哈希 |
| 多浏览器共享密钥 | `signCount` 差异可检测；必要时要求 `UV` 标记 (User Verified) 必须为 1 |

---

## 6. 兼容性

- 主流平台均支持 ES256 (P-256)。Ed25519 WebAuthn 支持度低，可暂不实现。
- `origin` 必须是 HTTPS 或 `localhost`。确保合约中的 `rpIdHash` 与生产域名一致。

---

## 7. 待办事项

- [ ] 设计并实现链下打包/解析工具函数（TypeScript SDK）。
- [ ] Move 模块 `webauthn_validator.move` 及单元测试。
- [ ] 计数器与 Credential ID 映射的全局存储结构。
- [ ] 文档补充：错误码定义、示例交易构造。
