// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

import { bcs } from '../bcs/index.js'
import { Bytes } from '../types/index.js'
import { bytes, sha256, toHEX, concatBytes, varintByteNum } from '../utils/index.js'

import { Signer } from './signer.js'
import { SIGNATURE_SCHEME_TO_FLAG } from './signatureScheme.js'
import {
  SigningEnvelope,
  EnvelopeMessageBuilder,
  RawTxHashEnvelope,
  BitcoinMessageEnvelope,
  WebAuthnEnvelope,
  WebauthnEnvelopeData,
  WebAuthnEnvelopeBuilder,
} from './envelope.js'

const BitcoinMessagePrefix = '\u0018Bitcoin Signed Message:\n'
const MessageInfoPrefix = 'Rooch Transaction:\n'

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

  /**
   * Create session authenticator with envelope support (v2 format)
   * @param txHash - Transaction hash to sign
   * @param signer - Signer instance
   * @param envelope - Envelope message builder (optional, defaults to RawTxHash)
   */
  static async sessionWithEnvelope(
    txHash: Bytes,
    signer: Signer,
    envelope?: EnvelopeMessageBuilder,
  ): Promise<Authenticator> {
    // Default to RawTxHash envelope for backward compatibility
    const envelopeBuilder = envelope || new RawTxHashEnvelope()
    const envelopeType = envelopeBuilder.getEnvelopeType()

    // Handle signing based on envelope type
    let signature: Bytes
    let messageData: Bytes | null = null
    const pubKeyBytes = signer.getPublicKey().toBytes()
    const schemeFlag = SIGNATURE_SCHEME_TO_FLAG[signer.getKeyScheme()]

    if (envelopeType === SigningEnvelope.RawTxHash) {
      // v1 format: sign directly over tx_hash
      signature = await signer.sign(txHash)
    } else if (envelopeType === SigningEnvelope.BitcoinMessageV0) {
      // Bitcoin message envelope
      const bitcoinEnvelope = envelope as BitcoinMessageEnvelope
      const messageToSign = bitcoinEnvelope.computeDigest(txHash)
      signature = await signer.sign(messageToSign)
      messageData = bitcoinEnvelope.buildMessage(txHash)
    } else if (envelopeType === SigningEnvelope.WebAuthnV0) {
      // WebAuthn envelope - special handling
      // Check if it's the new WebAuthnEnvelopeBuilder or legacy WebAuthnEnvelope
      if (envelope instanceof WebAuthnEnvelopeBuilder) {
        // New WebAuthnEnvelopeBuilder: signature is already available
        const webauthnEnvelopeBuilder = envelope as WebAuthnEnvelopeBuilder
        signature = webauthnEnvelopeBuilder.getSignature()
        messageData = webauthnEnvelopeBuilder.buildMessage(txHash)
      } else {
        // Legacy WebAuthnEnvelope: use old flow
        const webauthnEnvelope = envelope as WebAuthnEnvelope
        const messageToSign = webauthnEnvelope.computeDigest(txHash)
        signature = await signer.sign(messageToSign)
        messageData = this.encodeWebAuthnPayload(signer, webauthnEnvelope)
      }
    } else {
      throw new Error(`Unsupported envelope type: ${envelopeType}`)
    }

    // Build payload based on format
    let payload: Bytes
    if (envelopeType === SigningEnvelope.RawTxHash) {
      // v1 format: scheme | signature | public_key
      payload = new Uint8Array(1 + signature.length + pubKeyBytes.length)
      payload.set([schemeFlag])
      payload.set(signature, 1)
      payload.set(pubKeyBytes, 1 + signature.length)
    } else {
      // v2 format: scheme | envelope | signature | public_key | [message_len | message]
      const messageBytes = messageData || new Uint8Array(0)
      const hasMessage = messageBytes.length > 0
      const messageLenBytes = hasMessage ? varintByteNum(messageBytes.length) : new Uint8Array(0)

      const totalLength =
        1 + 1 + signature.length + pubKeyBytes.length + messageLenBytes.length + messageBytes.length

      payload = new Uint8Array(totalLength)
      let offset = 0

      payload.set([schemeFlag], offset)
      offset += 1
      payload.set([envelopeType], offset)
      offset += 1
      payload.set(signature, offset)
      offset += signature.length
      payload.set(pubKeyBytes, offset)
      offset += pubKeyBytes.length

      if (hasMessage) {
        payload.set(messageLenBytes, offset)
        offset += messageLenBytes.length
        payload.set(messageBytes, offset)
      }
    }

    return new Authenticator(BuiltinAuthValidator.SESSION, payload)
  }

  /**
   * Helper method to encode WebAuthn envelope data for BCS serialization
   */
  private static encodeWebAuthnPayload(_signer: Signer, envelope: WebAuthnEnvelope): Bytes {
    const authenticatorData = envelope.getAuthenticatorData()
    const clientDataJson = envelope.getClientDataJson()

    // Use the WebauthnEnvelopeData class for proper BCS encoding
    // Note: signature and public_key are now handled in the outer payload layer
    const webauthnEnvelopeData = new WebauthnEnvelopeData(authenticatorData, clientDataJson)

    return webauthnEnvelopeData.encode()
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
    envelope: SigningEnvelope = SigningEnvelope.RawTxHash,
  ): Promise<Authenticator> {
    const payload = await DIDAuthenticator.sign(txHash, signer, vmFragment, envelope)
    return new Authenticator(BuiltinAuthValidator.DID, payload)
  }

  static async didBitcoinMessage(
    txHash: Bytes,
    signer: Signer,
    vmFragment: string,
  ): Promise<Authenticator> {
    return Authenticator.did(txHash, signer, vmFragment, SigningEnvelope.BitcoinMessageV0)
  }
}

export interface DIDAuthPayload {
  scheme: number
  envelope: number
  vmFragment: string
  signature: Bytes
  message?: Bytes
}

export class DIDAuthenticator {
  static async sign(
    txHash: Bytes,
    signer: Signer,
    vmFragment: string,
    envelope: SigningEnvelope = SigningEnvelope.RawTxHash,
  ): Promise<Bytes> {
    let digest: Bytes
    let message: Bytes | undefined

    // Compute digest based on envelope type
    switch (envelope) {
      case SigningEnvelope.RawTxHash:
        digest = txHash
        break
      case SigningEnvelope.BitcoinMessageV0:
        const bitcoinMessage = new BitcoinSignMessage(txHash, MessageInfoPrefix + toHEX(txHash))
        digest = bitcoinMessage.hash()
        message = bytes('utf8', bitcoinMessage.raw())
        break
      case SigningEnvelope.WebAuthnV0:
        throw new Error('WebAuthn envelope not yet implemented for DID authenticator')
      default:
        throw new Error(`Unsupported envelope type: ${envelope}`)
    }

    const signature = await signer.sign(digest)
    const scheme = SIGNATURE_SCHEME_TO_FLAG[signer.getKeyScheme()]

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
