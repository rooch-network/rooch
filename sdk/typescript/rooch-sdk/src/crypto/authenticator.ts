// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

import { bcs } from '../bcs/index.js'
import { Bytes } from '../types/index.js'
import { bytes, sha256, toHEX, concatBytes, varintByteNum } from '../utils/index.js'

import { Signer } from './signer.js'
import { SIGNATURE_SCHEME_TO_FLAG } from './signatureScheme.js'

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
}
