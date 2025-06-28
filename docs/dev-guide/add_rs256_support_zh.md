# 在 Rooch 中支持 `rs256` (RSASSA-PKCS1-V1_5 with SHA-256) 签名算法方案

## 1. 目标

为了支持更多 WebAuthn 设备和应用，需要在 Rooch 框架中增加对 `rs256` 签名算法的支持。`rs256` 是一种使用 RSA (RSASSA-PKCS1-V1_5) 算法和 SHA-256 哈希函数的签名方案。

## 2. 方案概览

本次升级将遵循 Rooch 现有的加密模块扩展模式，涉及以下几个核心方面：
1.  **新增 `rs256` Move 模块**: 创建一个新的 Move 模块，用于声明与 `rs256` 相关的原生函数接口。
2.  **实现 `rs256` 原生函数**: 在 Rust 中使用 `rsa` 和 `sha2` crate 实现 `rs256` 的核心密码学操作，主要是签名验证。
3.  **集成到会话密钥验证**: 更新 `session_validator` 和 `session_key` 模块，使其能够识别、验证 `rs256` 签名，并从公钥生成认证密钥 (Authentication Key)。
4.  **集成到 DID 模块**: 更新 `did` 模块，以支持使用 `rs256` 密钥作为 DID 的验证方法 (Verification Method)，并能与会话密钥关联。
5.  **更新构建系统与依赖**: 将新的原生模块和 `rsa`、`sha2` Rust crate 添加到项目中。

## 3. 详细设计

### 3.1. 新增 Rust 依赖

*   在 `frameworks/rooch-framework/Cargo.toml` 的 `[dependencies]` 部分添加 `rsa` 和 `sha2` crate。

```toml
[dependencies]
#... 其他依赖
rsa = "0.9.8"
sha2 = "0.10.9"
```

### 3.2. `crypto/rs256.move` 模块

*   创建新文件 `frameworks/rooch-framework/sources/crypto/rs256.move`。
*   此模块将声明 `verify` 和 `verify_prehash` 原生函数。
*   定义相关常量，如 Modulus 长度、Exponent 长度、SHA-256 消息长度等。

```move
// frameworks/rooch-framework/sources/crypto/rs256.move
module rooch_framework::rs256 {
    /// 公钥 Modulus 长度
    const RSASSA_PKCS1_V1_5_MINIMUM_MODULUS_LENGTH: u64 = 2048;  // 2048 位 RSA 的 Modulus 长度 (Bits)
    /// 公钥 Exponent 长度
    const RSASSA_PKCS1_V1_5_MINIMUM_EXPONENT_LENGTH: u64 = 1; // 1 位 RSA 的 Exponent 长度
    const RSASSA_PKCS1_V1_5_MAXIMUM_EXPONENT_LENGTH: u64 = 512; // 512 位 RSA 的 Exponent 长度
    /// SHA2-256 消息长度
    const SHA256_MESSAGE_LENGTH: u64 = 32; // 32 位用于 RSA 校验的哈希后的消息长度

    // 错误代码
    const ErrorInvalidSignature: u64 = 1;
    const ErrorInvalidPubKey: u64 = 2;
    const ErrorInvalidHashType: u64 = 3;
    const ErrorInvalidMessageLength: u64 = 4;

    // 哈希函数
    const SHA256: u8 = 0;

    public fun sha256(): u8 {
        SHA256
    }

    /// 验证 RS256 签名。
    /// 消息未进行哈希。
    public fun verify(
        signature: &vector<u8>,
        n: &vector<u8>,
        e: &vector<u8>,
        msg: &vector<u8>
    ): bool;

    /// 验证 RS256 签名。
    /// 消息在验证前使用 SHA-256 进行哈希。
    public fun verify_prehash(
        signature: &vector<u8>,
        n: &vector<u8>,
        e: &vector<u8>,
        msg: &vector<u8>,
        hash_type: u8
    ): bool;
}
```

### 3.3. 原生实现 `crypto/rs256.rs`

*   创建新文件 `frameworks/rooch-framework/src/natives/rooch_framework/crypto/rs256.rs`。
*   实现 `native_verify` 函数和 `native_verify_prehash` 函数。
*   该函数将使用 `fastcrypto` crate：
    1.  使用 `fastcrypto::rsa::RSAPublicKey::from_raw_components` 从 modulus (n) 和 exponent (e) 的字节解析公钥。
    2.  使用 `rsa::padding::SignatureScheme` 和 `rsa::Pkcs1v15Sign` 进行签名验证。
    3.  `native_verify` 对原始消息 `msg` 应用 `Sha256` 哈希， `native_verify_prehash` 应用经 `Sha256` 哈希后的消息。
    4.  调用 `verify` 或 `verify_prehash` 函数进行验签。
    5.  返回布尔结果。

### 3.4. 构建系统集成

*   在 `frameworks/rooch-framework/src/natives/rooch_framework/mod.rs` 中，导入 `rs256` 模块，并将其原生函数集 `make_all` 的结果添加到 `ROOCH_FRAMEWORK_NATIVES` 列表中。

### 3.5. `session_key` & `session_validator` 模块更新

*   **`session_key.move`**:
    *   新增签名方案常量：`const SIGNATURE_SCHEME_RS256: u8 = 3;` (假设值为3)。
    *   新增函数 `rs256_public_key_to_authentication_key`。它接收 `rs256` 公钥，通过 `sha2-256` 哈希后，在前面加上 `SIGNATURE_SCHEME_RS256` 作为 scheme 字节，生成最终的 `authentication_key`。
*   **`auth_validator/session_validator.move`**:
    *   在 `validate_authenticator_payload` 和 `validate_signature` 函数中增加新的 `else if` 分支来处理 `SIGNATURE_SCHEME_RS256`。
    *   当 `scheme` 匹配时：
        *   调用 `rs256::verify` 或 `rs256::verify_prehash` 进行验签。
        *   调用 `session_key::rs256_public_key_to_authentication_key` 生成 `auth_key`。

### 3.6. `did.move` 模块更新

*   新增验证方法类型常量 `const VERIFICATION_METHOD_TYPE_RS256: vector<u8> = b"RsaVerificationKey2018";`。
*   更新 `internal_ensure_session_key`（或创建一个新的通用函数 `internal_ensure_session_key_generic`），以处理新的密钥类型。这将涉及：
    *   新增 `multibase::decode_rs256_key` (如果需要)。
    *   根据密钥类型调用对应的 `session_key::*_public_key_to_authentication_key` 函数。
*   新增 `add_rs256_authentication_method` 函数，保持与 `ed25519`， `secp256k1` 和 `ecdsa_r1` 类似的设计模式。
*   更新 `find_verification_method_by_session_key`，使其能够通过 `rs256` 密钥派生的 `session_key` 找到对应的验证方法。

## 4. 实施步骤

1.  **(文档)** 创建此方案文档。 (已完成)
2.  **(Rust)** 在 `Cargo.toml` 中添加 `rsa` 和 `fastcrypto` 依赖。 (已完成)
3.  **(Move)** 创建 `sources/crypto/rs256.move` 并声明原生函数。 (已完成)
4.  **(Rust)** 创建并实现 `src/natives/rooch_framework/crypto/rs256.rs`。 (已完成)
5.  **(Rust)** 在 `src/natives/rooch_framework/mod.rs` 中注册新的 `rs256` 原生模块。 (已完成)
6.  **(Move)** 修改 `sources/session_key.move` 添加 `rs256` 支持。
7.  **(Move)** 修改 `sources/auth_validator/session_validator.move` 添加 `rs256` 验证逻辑。
8.  **(Move)** 修改 `sources/did.move` 添加对 `rs256` 验证方法的支持。
9.  **(测试)** 编写必要的单元测试和集成测试，确保 `rs256` 在会话密钥和 DID 场景下功能正确。
