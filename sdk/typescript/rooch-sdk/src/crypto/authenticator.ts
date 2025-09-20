// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

import { bcs } from '../bcs/index.js'
import { Bytes } from '../types/index.js'
import { bytes, sha256, toHEX, concatBytes, varintByteNum } from '../utils/index.js'

import { Signer } from './signer.js'
import { SIGNATURE_SCHEME_TO_FLAG } from './signatureScheme.js'
import {
  SigningEnvelope,
  WebauthnEnvelopeData,
  WebAuthnUtils,
  type WebAuthnAssertionData,
} from './envelope.js'
import { BitcoinAddress } from '../address/index.js'

const BitcoinMessagePrefix = '\u0018Bitcoin Signed Message:\n'
const MessageInfoPrefix = 'Rooch Transaction:\n'

// Extended Signer interfaces for different wallet types

/**
 * Bitcoin wallet signer that automatically adds Bitcoin message prefix
 */
export interface BitcoinWalletSigner extends Signer {
  readonly autoPrefix: true
  getBitcoinAddress(): BitcoinAddress
}

/**
 * WebAuthn signer that provides assertion-based signing
 */
export interface WebAuthnSigner extends Signer {
  signAssertion(challenge: Bytes): Promise<WebAuthnAssertionData>
}

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

export class BitcoinSignMessage {
  readonly messagePrefix: string
  readonly messageInfo: string
  readonly txHash: Bytes

  constructor(txData: Bytes, messageInfo: string) {
    this.messagePrefix = BitcoinMessagePrefix
    let msg = messageInfo.startsWith(MessageInfoPrefix)
      ? messageInfo
      : MessageInfoPrefix + messageInfo

    msg = msg.charAt(msg.length - 1) !== '\n' ? msg + '\n' : msg

    this.messageInfo = msg

    this.txHash = txData
  }

  raw(): string {
    return this.messageInfo + toHEX(this.txHash)
  }

  encode(): Bytes {
    const msgHex = bytes('utf8', toHEX(this.txHash))
    // Convert each part to Uint8Array
    const infoBytes = bytes('utf8', this.messageInfo)
    const prefixBytes = concatBytes(
      bytes('utf8', this.messagePrefix),
      varintByteNum(infoBytes.length + msgHex.length),
    )

    // Calculate the total length
    const totalLength = prefixBytes.length + infoBytes.length + msgHex.length

    // Create a new Uint8Array to hold all the data
    const data = new Uint8Array(totalLength)

    // Copy each part into the new array
    let offset = 0
    data.set(prefixBytes, offset)
    offset += prefixBytes.length
    data.set(infoBytes, offset)
    offset += infoBytes.length
    data.set(msgHex, offset)

    return data
  }

  hash(): Bytes {
    return sha256(this.encode())
  }
}

export enum BuiltinAuthValidator {
  // ROOCH is a alias of SESSION, for backward compatibility
  ROOCH = 0x00,
  SESSION = 0x00,
  BITCOIN = 0x01,
  BITCOIN_MULTISIGN = 0x02,
  WEBAUTHN = 0x03,
  DID = 0x04,
}

export class Authenticator {
  readonly authValidatorId: number
  readonly payload: Bytes

  public constructor(authValidatorId: number, payload: Bytes) {
    this.authValidatorId = authValidatorId
    this.payload = payload
  }

  encode(): Bytes {
    return bcs.Authenticator.serialize({
      authValidatorId: this.authValidatorId,
      payload: this.payload,
    }).toBytes()
  }

  static async rooch(input: Bytes, signer: Signer) {
    return this.session(input, signer)
  }

  static async session(input: Bytes, signer: Signer) {
    const signature = await signer.sign(input)
    const pubKeyBytes = signer.getPublicKey().toBytes()
    const serializedSignature = new Uint8Array(1 + signature.length + pubKeyBytes.length)
    serializedSignature.set([SIGNATURE_SCHEME_TO_FLAG[signer.getKeyScheme()]])
    serializedSignature.set(signature, 1)
    serializedSignature.set(signer.getPublicKey().toBytes(), 1 + signature.length)

    return new Authenticator(BuiltinAuthValidator.SESSION, serializedSignature)
  }

  static async bitcoin(
    input: BitcoinSignMessage,
    signer: Signer,
    signWith: 'hash' | 'raw' = 'hash',
  ): Promise<Authenticator> {
    if (!input.messageInfo.startsWith(MessageInfoPrefix)) {
      throw Error('invalid message info')
    }

    const messageLength = bytes('utf8', input.messageInfo).length + toHEX(input.txHash).length
    const sign = await signer.sign(signWith === 'hash' ? input.hash() : bytes('utf8', input.raw()))

    const payload = bcs.BitcoinAuthPayload.serialize({
      signature: sign,
      messagePrefix: concatBytes(bytes('utf8', input.messagePrefix), varintByteNum(messageLength)),
      messageInfo: bytes('utf8', input.messageInfo),
      publicKey: signer.getPublicKey().toBytes(),
      fromAddress: bytes('utf8', signer.getBitcoinAddress().toStr()),
    }).toBytes()

    return new Authenticator(BuiltinAuthValidator.BITCOIN, payload)
  }

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

export interface DIDAuthPayload {
  envelope: number
  vmFragment: string
  signature: string | Uint8Array
  message: string | Uint8Array | null | undefined
}

export class DIDAuthenticator {
  static async sign(
    txHash: Bytes,
    signer: Signer,
    vmFragment: string,
    envelope: SigningEnvelope = SigningEnvelope.RawTxHash,
  ): Promise<Bytes> {
    let signature: Bytes
    let message: Bytes | null = null

    // Handle different envelope types with signer-specific logic
    switch (envelope) {
      case SigningEnvelope.RawTxHash:
        signature = await signer.sign(txHash)
        break

      case SigningEnvelope.BitcoinMessageV0:
        if (isBitcoinWalletSigner(signer)) {
          // Bitcoin wallet will automatically add prefix, just pass the message content
          const template = MessageInfoPrefix + toHEX(txHash)
          signature = await signer.sign(bytes('utf8', template))
          message = bytes('utf8', template)
        } else {
          // Regular signer needs manual Bitcoin message construction
          const template = MessageInfoPrefix + toHEX(txHash)
          const bitcoinMessage = new BitcoinSignMessage(txHash, template)
          signature = await signer.sign(bitcoinMessage.hash())
          // Move expects the raw message content (without Bitcoin prefix)
          message = bytes('utf8', template)
        }
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
        break

      default:
        throw new Error(`Unsupported envelope type: ${envelope}`)
    }

    const payload: DIDAuthPayload = {
      envelope,
      vmFragment,
      signature,
      message,
    }

    return bcs.DIDAuthPayload.serialize(payload).toBytes()
  }

  static async signWebAuthn(
    txHash: Bytes,
    vmFragment: string,
    assertionData: WebAuthnAssertionData,
  ): Promise<Bytes> {
    // Validate that the challenge matches the transaction hash
    if (!WebAuthnUtils.validateChallenge(assertionData.clientDataJSON, txHash)) {
      throw new Error('WebAuthn challenge does not match transaction hash')
    }

    // Create the WebAuthn envelope data
    const webauthnEnvelopeData = new WebauthnEnvelopeData(
      assertionData.authenticatorData,
      assertionData.clientDataJSON,
    )
    const message = webauthnEnvelopeData.encode()

    // Use the raw signature from the assertion data
    const signature = assertionData.rawSignature

    const payload: DIDAuthPayload = {
      envelope: SigningEnvelope.WebAuthnV0,
      vmFragment,
      signature,
      message,
    }

    return bcs.DIDAuthPayload.serialize(payload).toBytes()
  }
}
