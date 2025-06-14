# 在 Rooch 中支持 `ecdsa_r1` (Secp256r1) 方案

## 1. 目标

为了支持更多 WebAuthn 设备（这些设备普遍使用 P-256/Secp256r1 曲线），需要在 Rooch 框架中增加对 `ecdsa_r1` 签名算法的支持。现有的 `ecdsa_k1` (Secp256k1) 和 `ed25519` 的实现将作为本次开发的主要参考。

## 2. 方案概览

本次升级将遵循 Rooch 现有的加密模块扩展模式，涉及以下几个核心方面：
1.  **新增 `ecdsa_r1` Move 模块**: 创建一个新的 Move 模块，用于声明与 `ecdsa_r1` 相关的原生函数接口。
2.  **实现 `ecdsa_r1` 原生函数**: 在 Rust 中使用 `p256` crate 实现 `ecdsa_r1` 的核心密码学操作，主要是签名验证。
3.  **集成到会话密钥验证**: 更新 `session_validator` 和 `session_key` 模块，使其能够识别、验证 `ecdsa_r1` 签名，并从公钥生成认证密钥 (Authentication Key)。
4.  **集成到 DID 模块**: 更新 `did` 模块，以支持使用 `ecdsa_r1` 密钥作为 DID 的验证方法 (Verification Method)，并能与会话密钥关联。
5.  **更新构建系统与依赖**: 将新的原生模块和 `p256` Rust crate 添加到项目中。

## 3. 详细设计

### 3.1. 新增 Rust 依赖

*   在 `frameworks/rooch-framework/Cargo.toml` 的 `[dependencies]` 部分添加 `p256` crate。我们将启用 `ecdsa` 特性以获得签名验证功能。

```toml
[dependencies]
# ... other dependencies
p256 = { version = "0.13", features = ["ecdsa"] }
sha2 = "0.10.6"
signature = "2.2.0"
```

### 3.2. `crypto/ecdsa_r1.move` 模块

*   创建新文件 `frameworks/rooch-framework/sources/crypto/ecdsa_r1.move`。
*   此模块将声明 `verify` 原生函数，其接口设计将与 `ecdsa_k1::verify` 保持一致。
*   定义相关常量，如公钥长度、签名长度等。

```move
// frameworks/rooch-framework/sources/crypto/ecdsa_r1.move
module rooch_framework::ecdsa_r1 {
    /// Compressed public key length for P-256
    const ECDSA_R1_COMPRESSED_PUBKEY_LENGTH: u64 = 33;
    /// Signature length (r, s)
    const ECDSA_R1_SIGNATURE_LENGTH: u64 = 64;

    // Error codes
    const ErrorInvalidSignature: u64 = 1;
    const ErrorInvalidPubKey: u64 = 2;

    /// Verifies an ECDSA signature over the secp256r1 (P-256) curve.
    /// The message is hashed with SHA256 before verification.
    native public fun verify(
        signature: &vector<u8>,
        public_key: &vector<u8>,
        msg: &vector<u8>
    ): bool;

    public fun public_key_length(): u64 {
        ECDSA_R1_COMPRESSED_PUBKEY_LENGTH
    }
}
```

### 3.3. 原生实现 `crypto/ecdsa_r1.rs`

*   创建新文件 `frameworks/rooch-framework/src/natives/rooch_framework/crypto/ecdsa_r1.rs`。
*   实现 `native_verify` 函数。
*   该函数将使用 `p256` crate：
    1.  使用 `p256::ecdsa::VerifyingKey::from_sec1_bytes` 从字节解析公钥。
    2.  使用 `p256::ecdsa::Signature::from_bytes` 从 `(r, s)` 字节解析签名。
    3.  对原始消息 `msg` 应用 `SHA256` 哈希。
    4.  调用 `key.verify_prehashed(&hashed_msg, &sig)` 进行验签。
    5.  返回布尔结果。

### 3.4. 构建系统集成

*   在 `frameworks/rooch-framework/src/natives/rooch_framework/mod.rs` 中，导入 `ecdsa_r1` 模块，并将其原生函数集 `make_all` 的结果添加到 `ROOCH_FRAMEWORK_NATIVES` 列表中。

### 3.5. `session_key` & `session_validator` 模块更新

*   **`session_key.move`**:
    *   新增签名方案常量：`const SIGNATURE_SCHEME_ECDSAR1: u8 = 2;` (假设值为2)。
    *   新增函数 `secp256r1_public_key_to_authentication_key`。它接收 `r1` 公钥，通过 `sha2-256` 哈希后，在前面加上 `SIGNATURE_SCHEME_ECDSAR1` 作为 scheme 字节，生成最终的 `authentication_key`。
*   **`auth_validator/session_validator.move`**:
    *   在 `validate_authenticator_payload` 和 `validate_signature` 函数中增加新的 `else if` 分支来处理 `SIGNATURE_SCHEME_ECDSAR1`。
    *   当 `scheme` 匹配时：
        *   调用 `ecdsa_r1::verify` 进行验签。
        *   调用 `session_key::secp256r1_public_key_to_authentication_key` 生成 `auth_key`。

### 3.6. `did.move` 模块更新

*   新增验证方法类型常量 `const VERIFICATION_METHOD_TYPE_SECP256R1: vector<u8> = b"EcdsaSecp256r1VerificationKey2019";`。
*   更新 `internal_ensure_session_key`（或创建一个新的通用函数 `internal_ensure_session_key_generic`），以处理新的密钥类型。这将涉及：
    *   新增 `multibase::decode_secp256r1_key` (如果需要)。
    *   根据密钥类型调用对应的 `session_key::*_public_key_to_authentication_key` 函数。
*   新增 `add_secp256r1_authentication_method` 函数，保持与 `ed25519` 和 `secp256k1` 类似的设计模式。
*   更新 `find_verification_method_by_session_key`，使其能够通过 `r1` 密钥派生的 `session_key` 找到对应的验证方法。

## 4. 实施步骤

1.  **(文档)** 创建此方案文档。 (已完成)
2.  **(Rust)** 在 `Cargo.toml` 中添加 `p256`, `sha2`, `signature` 依赖。
3.  **(Move)** 创建 `sources/crypto/ecdsa_r1.move` 并声明原生函数。
4.  **(Rust)** 创建并实现 `src/natives/rooch_framework/crypto/ecdsa_r1.rs`。
5.  **(Rust)** 在 `src/natives/rooch_framework/mod.rs` 中注册新的 `ecdsa_r1` 原生模块。
6.  **(Move)** 修改 `sources/session_key.move` 添加 `r1` 支持。
7.  **(Move)** 修改 `sources/auth_validator/session_validator.move` 添加 `r1` 验证逻辑。
8.  **(Move)** 修改 `sources/did.move` 添加对 `r1` 验证方法的支持。
9.  **(测试)** 编写必要的单元测试和集成测试，确保 `ecdsa_r1` 在会话密钥和 DID 场景下功能正确。 