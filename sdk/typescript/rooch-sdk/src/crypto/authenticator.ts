// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

import { bcs } from '@/bcs'
import { Bytes } from '@/types'
import { bytes, sha256, toHEX } from '@/utils'

import { Signer } from './signer'
import { SIGNATURE_SCHEME_TO_FLAG } from './signatureScheme'

const BitcoinMessagePrefix = '\x18Bitcoin Signed Message:\n'
const MessageInfoPrefix = 'Rooch Transaction:\n'

export class BitcoinSignMessage {
  readonly messagePrefix: string
  readonly messageInfo: string
  readonly txHash: Bytes

  constructor(txData: Bytes, messageInfo: string) {
    this.messagePrefix = BitcoinMessagePrefix
    this.messageInfo = messageInfo.startsWith(MessageInfoPrefix)
      ? messageInfo
      : MessageInfoPrefix + messageInfo
    this.txHash = txData
  }

  display(): string {
    return this.messageInfo + this.txHash
  }

  encode(): Bytes {
    const msgHex = bytes('utf8', toHEX(this.txHash))
    // Convert each part to Uint8Array
    const prefixBytes = bytes('utf8', this.messagePrefix)
    const infoBytes = bytes('utf8', this.messageInfo)

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

    // Avoid the 255 length limit
    if (data.length > 255) {
      throw Error(`message info length cannot be greater than > ${data.length - msgHex.length}`)
    }

    return data
  }

  hash(): Bytes {
    return sha256(this.encode())
  }
}

export enum BuiltinAuthValidator {
  ROOCH = 0x00,
  BITCOIN = 0x01,
  // ETHEREUM= 0x02
}

export class Authenticator {
  readonly authValidatorId: number
  readonly payload: Bytes

  private constructor(authValidatorId: number, payload: Bytes) {
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
    const signature = await signer.sign(input)
    const pubKeyBytes = signer.getPublicKey().toBytes()
    const serializedSignature = new Uint8Array(1 + signature.length + pubKeyBytes.length)
    serializedSignature.set([SIGNATURE_SCHEME_TO_FLAG[signer.getKeyScheme()]])
    serializedSignature.set(signature, 1)
    serializedSignature.set(signer.getPublicKey().toBytes(), 1 + signature.length)

    return new Authenticator(BuiltinAuthValidator.ROOCH, serializedSignature)
  }

  static async bitcoin(input: BitcoinSignMessage, signer: Signer): Promise<Authenticator> {
    if (!input.messageInfo.startsWith(MessageInfoPrefix)) {
      throw Error('invalid message info')
    }

    const sign = await signer.sign(input.hash())

    const payload = bcs.BitcoinAuthPayload.serialize({
      signature: sign,
      messagePrefix: input.messagePrefix,
      messageInfo: input.messageInfo,
      publicKey: signer.getPublicKey().toBytes(),
      fromAddress: bytes('utf8', signer.getBitcoinAddress().toStr()),
    }).toBytes()

    return new Authenticator(BuiltinAuthValidator.BITCOIN, payload)
  }
}
