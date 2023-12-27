// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

import { ChainInfo, SerializedSignature } from '@roochnetwork/rooch-sdk'
import { sha3_256 } from '@noble/hashes/sha3'
import { Buffer } from 'buffer'
import { WalletAccount } from '../WalletAccount'

export abstract class BaseWallet {
  protected abstract sign(msg: string): Promise<string>
  abstract getTarget(): any
  abstract getScheme(): number

  // TODO: What happens if the user rejects the request
  abstract connect(chainInfo: ChainInfo): Promise<WalletAccount[]>

  async signMessage(msg: Uint8Array) {
    const digest = sha3_256(msg)
    return await this.signMessageWithHashed(digest)
  }

  async signMessageWithHashed(msg: Uint8Array) {
    // TODO: fix with rooch-sdk hexString class
    let hex = Array.from(msg)
      .map((b) => b.toString(16).padStart(2, '0'))
      .join('')

    return this.toSerializedSignature(await this.sign(hex))
  }

  async checkInstalled(): Promise<boolean> {
    for (let i = 1; i < 10 && !this.getTarget(); i += 1) {
      await new Promise((resolve) => setTimeout(resolve, 100 * i))
    }

    return Promise.resolve(this.getTarget() !== undefined)
  }

  normalize_recovery_id(v: number) {
    let normalizeV = v - 27 - 4

    if (normalizeV < 0) {
      normalizeV = normalizeV + 4
    }

    return normalizeV
  }

  private toSerializedSignature(signature: string): SerializedSignature {
    let signBuffer = Buffer.from(signature, 'base64')

    const normalizeSignBuffer = Buffer.concat([
      signBuffer.subarray(1),
      Buffer.from([this.normalize_recovery_id(signBuffer[0])]),
    ])

    // TODO: add address
    const serializedSignature = new Uint8Array(normalizeSignBuffer.length)
    serializedSignature.set(normalizeSignBuffer)

    return serializedSignature
  }
}
