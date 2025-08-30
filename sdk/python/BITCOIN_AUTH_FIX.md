# Bitcoin Authentication Address Mapping Fix

## 问题描述

用户报告："现在集成测试的签名以及 btc 地址都能验证过去了,但发现 sender 和 btc地址得到的 rooch 地址不一致"

这个问题的根本原因是 Python SDK 中存在两种不同的 Rooch 地址生成方式：

1. **KeyPair 直接生成的 Rooch 地址** (Python SDK 错误实现)：
   - 使用 `SHA256(uncompressed_public_key)` 
   - 由 `KeyPair.get_rooch_address()` 生成
   - **这是错误的实现方式！**

2. **正确的 Rooch 地址生成方式** (应该与 Rust/TypeScript/Move 一致)：
   - 从公钥生成 Bitcoin 地址（Taproot P2TR）
   - 使用 `Blake2b256(bitcoin_address.bytes)` 生成 Rooch 地址
   - 这与 Move 的 `bitcoin_address::to_rooch_address()` 实现一致

### 正确的流程

根据 Rust 和 TypeScript SDK 的实现，正确的流程应该是：

```
PublicKey -> BitcoinAddress -> RoochAddress
```

- **Rust**: `PublicKey::rooch_address() -> bitcoin_address() -> to_rooch_address()`
- **TypeScript**: `PublicKey::toAddress() -> bitcoinAddress.genRoochAddress()`
- **Move**: `bitcoin_address::to_rooch_address(&bitcoin_addr)`

在使用 Bitcoin 验证器时，Move 代码期望 sender 是从 Bitcoin 地址转换而来的 Rooch 地址，但 Python SDK 错误地使用了直接从 KeyPair 生成的地址。

## 解决方案

### 临时修复 (当前实现)

我们实现了一个临时的自动修正机制：

1. **实现 Bitcoin 地址到 Rooch 地址转换** - 在 `bitcoin.py` 中添加了 `to_rooch_address()` 方法

2. **TransactionBuilder 自动修正** - 在签名时检测地址不匹配并自动更新 sender

3. **保持向后兼容** - 现有 API 无需修改，自动处理地址转换

### 根本问题需要重构修复

**真正的问题是**: Python SDK 的 `KeyPair.get_rooch_address()` 实现方式不正确。

**正确的实现应该是**:
```python
def get_rooch_address(self) -> RoochAddress:
    """Get Rooch address by: PublicKey -> BitcoinAddress -> RoochAddress"""
    # 1. 从公钥生成 Bitcoin 地址 (Taproot)
    compressed_pk = self._get_compressed_public_key()
    bitcoin_addr = BitcoinAddress.from_taproot_public_key(compressed_pk)
    
    # 2. 从 Bitcoin 地址生成 Rooch 地址
    return RoochAddress.from_hex(bitcoin_addr.to_rooch_address())
```

**当前的错误实现**:
```python
def get_rooch_address(self) -> RoochAddress:
    """错误的实现：直接对公钥进行 SHA256"""
    public_key = self.get_public_key()
    address_bytes = hashlib.sha256(public_key).digest()  # 这是错误的！
    return RoochAddress(address_bytes)
```

### 重构计划

1. **修复 KeyPair.get_rooch_address()** - 改为正确的 PublicKey -> BitcoinAddress -> RoochAddress 流程
2. **移除临时自动修正机制** - 不再需要在 TransactionBuilder 中特殊处理
3. **保持 API 兼容性** - 外部接口保持不变，只修改内部实现
4. **统一所有 SDK** - 确保 Python/Rust/TypeScript 的实现逻辑一致

## 测试验证

创建了多个测试文件验证修复：

1. **test_address_mapping_fix.py** - 全面测试地址映射修复
2. **test_integration_ready.py** - 验证集成测试准备就绪
3. **test_bitcoin_auth.py** - 完整的 Bitcoin 认证流程测试

### 测试结果示例

```
Original Rooch address (KeyPair): 0x3861a642f491685084224ec13154ef5def0ec872e1c161a34e16178923839335
Bitcoin-derived Rooch address:   0x8f45cc3e0b5e49b3c9640bde1aaac5d22565be0524caa7637be66f8efc8d99f0

✓ TransactionBuilder auto-corrected sender to Bitcoin-derived address!
✓ Bitcoin authentication should now work correctly
```

## 技术细节

### Move 兼容性

实现严格遵循 Move 代码的逻辑：

1. **Taproot 地址生成** - 使用 embit 库确保 BIP341 兼容性
2. **地址字节格式** - 匹配 Move 的 `BitcoinAddress.bytes` 结构
3. **Blake2b256 哈希** - 与 Move 的 `moveos_std::hash::blake2b256` 一致

### 向后兼容性

- 保持现有 API 不变
- 自动检测和修正，无需用户代码更改
- 支持两种认证方式（普通 Rooch 和 Bitcoin）

## 影响范围

这个修复解决了：

1. **Error 1010 (ErrorValidateInvalidAuthenticator)** - Bitcoin 验证器拒绝错误的 sender 地址
2. **集成测试失败** - Bitcoin 认证相关的测试现在应该能通过
3. **地址一致性** - sender 和 Bitcoin 地址现在生成一致的 Rooch 地址

**注意**: 当前的修复是**临时性的**，真正的解决方案需要重构 `KeyPair.get_rooch_address()` 方法。

## 使用说明

**当前临时修复**: 用户无需修改任何代码，`TransactionBuilder` 会自动处理地址一致性。

**未来重构后**: 
- `KeyPair.get_rooch_address()` 将返回正确的地址
- 无需特殊处理，所有 SDK 行为一致
- 更好的性能（无需重复计算地址转换）

## 技术债务

1. **临时自动修正机制需要移除** - 在 TransactionBuilder 中的特殊处理应该在重构后删除
2. **KeyPair.get_rooch_address() 需要重构** - 改为正确的 PublicKey -> BitcoinAddress -> RoochAddress 流程
3. **测试用例需要更新** - 确保测试覆盖正确的地址生成逻辑
4. **文档需要更新** - 说明正确的地址生成流程

这确保了 Bitcoin 认证的完整性和与 Move 代码的兼容性，同时为未来的正确重构奠定了基础。
