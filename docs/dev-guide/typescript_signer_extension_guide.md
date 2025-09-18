# TypeScript SDK Signer 扩展指南

## 概述

本文档描述了 Rooch TypeScript SDK 中 Signer 接口的扩展方案，以及如何通过类型判断来统一处理不同类型的签名器。这个设计允许不同的钱包和签名方式（如 Bitcoin 钱包、WebAuthn）与 DID authenticator 无缝集成。

## 背景

在实际应用中，不同类型的签名器有着不同的行为特征：

1. **Bitcoin 钱包**：会自动添加 Bitcoin message 前缀
2. **WebAuthn**：返回的不仅仅是签名，还包含 authenticator data 和 client data
3. **普通签名器**：直接对数据进行签名

为了统一处理这些差异，我们设计了基于类型判断的扩展方案。

## 核心设计

### 1. 扩展 Signer 接口

#### BitcoinWalletSigner 接口

```typescript
/**
 * Bitcoin wallet signer that automatically adds Bitcoin message prefix
 */
export interface BitcoinWalletSigner extends Signer {
  readonly autoPrefix: true
  getBitcoinAddress(): BitcoinAddress
}
```

**特点**：
- 具有 `autoPrefix: true` 标识
- 会自动为消息添加 Bitcoin message 前缀
- 提供 `getBitcoinAddress()` 方法

#### WebAuthnSigner 接口

```typescript
/**
 * WebAuthn signer that provides assertion-based signing
 */
export interface WebAuthnSigner extends Signer {
  signAssertion(challenge: Bytes): Promise<WebAuthnAssertionData>
}
```

**特点**：
- 提供 `signAssertion()` 方法返回完整的 WebAuthn 断言数据
- 支持 WebAuthn 的特殊签名流程

### 2. 类型守卫函数

```typescript
/**
 * Type guard to check if a signer is a Bitcoin wallet signer
 */
export function isBitcoinWalletSigner(signer: Signer): signer is BitcoinWalletSigner {
  return 'autoPrefix' in signer && (signer as any).autoPrefix === true
}

/**
 * Type guard to check if a signer is a WebAuthn signer
 */
export function isWebAuthnSigner(signer: Signer): signer is WebAuthnSigner {
  return 'signAssertion' in signer && typeof (signer as any).signAssertion === 'function'
}
```

### 3. 统一的 DIDAuthenticator

```typescript
export class DIDAuthenticator {
  static async sign(
    txHash: Bytes,
    signer: Signer,
    vmFragment: string,
    envelope: SigningEnvelope = SigningEnvelope.RawTxHash,
  ): Promise<Bytes> {
    let signature: Bytes
    let message: Bytes | null = null
    let scheme: number

    // Handle different envelope types with signer-specific logic
    switch (envelope) {
      case SigningEnvelope.RawTxHash:
        signature = await signer.sign(txHash)
        scheme = SIGNATURE_SCHEME_TO_FLAG[signer.getKeyScheme()]
        break

      case SigningEnvelope.BitcoinMessageV0:
        if (isBitcoinWalletSigner(signer)) {
          // Bitcoin wallet will automatically add prefix, just pass the message content
          const template = MessageInfoPrefix + toHEX(txHash)
          signature = await signer.sign(bytes('utf8', template))
          message = bytes('utf8', template)
        } else {
          // Regular signer needs manual Bitcoin message construction
          const bitcoinMessage = new BitcoinSignMessage(txHash, MessageInfoPrefix + toHEX(txHash))
          signature = await signer.sign(bitcoinMessage.hash())
          message = bytes('utf8', bitcoinMessage.raw())
        }
        scheme = SIGNATURE_SCHEME_TO_FLAG[signer.getKeyScheme()]
        break

      case SigningEnvelope.WebAuthnV0:
        if (!isWebAuthnSigner(signer)) {
          throw new Error('WebAuthn envelope requires a WebAuthnSigner')
        }
        
        // Use WebAuthn-specific signing method
        const assertionData = await signer.signAssertion(txHash)
        
        // Validate challenge
        if (!WebAuthnUtils.validateChallenge(assertionData.clientDataJSON, txHash)) {
          throw new Error('WebAuthn challenge does not match transaction hash')
        }
        
        // Build envelope data
        const webauthnEnvelopeData = new WebauthnEnvelopeData(
          assertionData.authenticatorData,
          assertionData.clientDataJSON,
        )
        
        signature = assertionData.rawSignature
        message = webauthnEnvelopeData.encode()
        scheme = SIGNATURE_SCHEME_TO_FLAG.EcdsaR1
        break

      default:
        throw new Error(`Unsupported envelope type: ${envelope}`)
    }

    const payload: DIDAuthPayload = {
      scheme,
      envelope,
      vmFragment,
      signature,
      message,
    }

    return bcs.DIDAuthPayload.serialize(payload).toBytes()
  }
}
```

### 4. 智能的 Authenticator API

```typescript
export class Authenticator {
  static async did(
    txHash: Bytes,
    signer: Signer,
    vmFragment: string,
    envelope?: SigningEnvelope,
  ): Promise<Authenticator> {
    // Auto-select envelope based on signer type if not specified
    if (envelope === undefined) {
      if (isWebAuthnSigner(signer)) {
        envelope = SigningEnvelope.WebAuthnV0
      } else if (isBitcoinWalletSigner(signer)) {
        envelope = SigningEnvelope.BitcoinMessageV0
      } else {
        envelope = SigningEnvelope.RawTxHash
      }
    }
    
    const payload = await DIDAuthenticator.sign(txHash, signer, vmFragment, envelope)
    return new Authenticator(BuiltinAuthValidator.DID, payload)
  }

  // 便捷方法
  static async didBitcoinMessage(
    txHash: Bytes,
    signer: Signer,
    vmFragment: string,
  ): Promise<Authenticator> {
    return this.did(txHash, signer, vmFragment, SigningEnvelope.BitcoinMessageV0)
  }

  static async didWebAuthn(
    txHash: Bytes,
    vmFragment: string,
    assertionData: WebAuthnAssertionData,
  ): Promise<Authenticator> {
    const payload = await DIDAuthenticator.signWebAuthn(txHash, vmFragment, assertionData)
    return new Authenticator(BuiltinAuthValidator.DID, payload)
  }
}
```

## 使用示例

### 1. 普通 Signer（自动选择 RawTxHash）

```typescript
const normalSigner = new Ed25519Signer(privateKey)
const auth = await Authenticator.did(txHash, normalSigner, vmFragment)
// 自动选择 RawTxHash envelope
```

### 2. Bitcoin 钱包 Signer（自动选择 BitcoinMessageV0）

```typescript
class MyBitcoinWalletSigner extends Signer implements BitcoinWalletSigner {
  readonly autoPrefix = true as const
  
  async sign(input: Bytes): Promise<Bytes> {
    // 钱包会自动添加 "Bitcoin Signed Message:\n" 前缀
    return this.wallet.signMessage(new TextDecoder().decode(input))
  }
  
  getBitcoinAddress(): BitcoinAddress {
    return this.wallet.getAddress()
  }
  
  // ... 其他必需方法
}

const bitcoinSigner = new MyBitcoinWalletSigner(wallet)
const auth = await Authenticator.did(txHash, bitcoinSigner, vmFragment)
// 自动选择 BitcoinMessageV0 envelope，正确处理前缀
```

### 3. WebAuthn Signer（自动选择 WebAuthnV0）

```typescript
class MyWebAuthnSigner extends Signer implements WebAuthnSigner {
  async signAssertion(challenge: Bytes): Promise<WebAuthnAssertionData> {
    const options: PublicKeyCredentialRequestOptions = {
      challenge,
      rpId: this.rpId,
      allowCredentials: this.credentialId ? [{ 
        id: fromB64(this.credentialId), 
        type: 'public-key' 
      }] : [],
      userVerification: 'preferred',
      timeout: 60000,
    }

    const credential = await navigator.credentials.get({ 
      publicKey: options 
    }) as PublicKeyCredential
    
    if (!credential) {
      throw new Error('No credential received')
    }

    const response = credential.response as AuthenticatorAssertionResponse
    return WebAuthnUtils.parseAssertionResponse(response)
  }
  
  async sign(input: Bytes): Promise<Bytes> {
    // 基础 sign 方法可以调用 signAssertion
    const assertionData = await this.signAssertion(input)
    return assertionData.rawSignature
  }
  
  // ... 其他必需方法
}

const webauthnSigner = new MyWebAuthnSigner(rpId, credentialId, publicKey)
const auth = await Authenticator.did(txHash, webauthnSigner, vmFragment)
// 自动选择 WebAuthnV0 envelope，使用 signAssertion 方法
```

### 4. 手动指定 Envelope

```typescript
// 强制使用特定的 envelope 类型
const auth = await Authenticator.did(
  txHash, 
  normalSigner, 
  vmFragment, 
  SigningEnvelope.BitcoinMessageV0
)
// 即使是普通 signer，也会使用 BitcoinMessageV0 envelope
```

### 5. 直接使用 WebAuthn 断言数据

```typescript
// 如果已经有 WebAuthn 断言数据
const assertion = await navigator.credentials.get({...})
const assertionData = WebAuthnUtils.parseAssertionResponse(assertion.response)
const auth = await Authenticator.didWebAuthn(txHash, vmFragment, assertionData)
```

## 扩展新的 Signer 类型

### 步骤 1：定义新的接口

```typescript
export interface MyCustomSigner extends Signer {
  // 定义特有的标识或方法
  readonly customFeature: true
  customSignMethod(data: Bytes): Promise<CustomSignatureData>
}
```

### 步骤 2：添加类型守卫

```typescript
export function isMyCustomSigner(signer: Signer): signer is MyCustomSigner {
  return 'customFeature' in signer && (signer as any).customFeature === true
}
```

### 步骤 3：扩展 DIDAuthenticator

```typescript
// 在 DIDAuthenticator.sign 的 switch 语句中添加新的 case
case SigningEnvelope.MyCustomV0:
  if (!isMyCustomSigner(signer)) {
    throw new Error('MyCustom envelope requires a MyCustomSigner')
  }
  
  const customData = await signer.customSignMethod(txHash)
  signature = customData.signature
  message = customData.message
  scheme = customData.scheme
  break
```

### 步骤 4：更新智能选择逻辑

```typescript
// 在 Authenticator.did 中更新自动选择逻辑
if (envelope === undefined) {
  if (isMyCustomSigner(signer)) {
    envelope = SigningEnvelope.MyCustomV0
  } else if (isWebAuthnSigner(signer)) {
    envelope = SigningEnvelope.WebAuthnV0
  } else if (isBitcoinWalletSigner(signer)) {
    envelope = SigningEnvelope.BitcoinMessageV0
  } else {
    envelope = SigningEnvelope.RawTxHash
  }
}
```

### 步骤 5：添加便捷方法

```typescript
static async didMyCustom(
  txHash: Bytes,
  signer: MyCustomSigner,
  vmFragment: string,
): Promise<Authenticator> {
  return this.did(txHash, signer, vmFragment, SigningEnvelope.MyCustomV0)
}
```

## 设计优势

1. **统一性**：所有 DID 认证都通过统一的 API
2. **智能性**：根据 Signer 类型自动选择最合适的 envelope
3. **扩展性**：新的 Signer 类型只需实现相应接口
4. **兼容性**：保持向后兼容，现有代码无需修改
5. **类型安全**：TypeScript 类型守卫确保正确的方法调用
6. **清晰的职责分离**：每种 Signer 封装自己的特殊逻辑

## 注意事项

1. **接口一致性**：新的 Signer 接口必须继承基础 `Signer` 类
2. **类型守卫准确性**：确保类型守卫函数能准确识别 Signer 类型
3. **错误处理**：为不匹配的 Signer 和 envelope 组合提供清晰的错误信息
4. **测试覆盖**：为每种新的 Signer 类型添加完整的测试用例
5. **文档更新**：及时更新 API 文档和使用示例

## 相关文件

- `src/crypto/authenticator.ts` - 主要实现文件
- `src/crypto/signer.ts` - 基础 Signer 类定义
- `src/crypto/envelope.ts` - Envelope 相关类型和工具
- `src/crypto/index.ts` - 导出配置
- `src/crypto/authenticator.test.ts` - 测试用例

这个扩展方案为 Rooch TypeScript SDK 提供了灵活且类型安全的 Signer 扩展机制，能够优雅地处理各种钱包和签名方式的差异。
