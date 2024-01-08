// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

import { sha3_256 } from '@noble/hashes/sha3'
import { WalletAccount } from '../WalletAccount'
import { SerializedSignature } from '@roochnetwork/rooch-sdk'

export abstract class BaseWallet {
  protected abstract sign(msg: string, fromAddress: string): Promise<string>
  protected abstract toSerializedSignature(
    signature: string,
    fromAddress: string,
  ): SerializedSignature

  abstract normalize_recovery_id(recoveryID: number): number
  abstract getTarget(): any
  abstract getScheme(): number

  abstract connect(): Promise<WalletAccount[]>

  async signMessage(msg: Uint8Array, fromAddress: string) {
    const digest = sha3_256(msg)
    return await this.signMessageWithHashed(digest, fromAddress)
  }

  async signMessageWithHashed(msg: Uint8Array, fromAddress: string) {
    // TODO: fix with rooch-sdk hexString class
    let hex = Array.from(msg)
      .map((b) => b.toString(16).padStart(2, '0'))
      .join('')

    return this.toSerializedSignature(await this.sign(hex, fromAddress), fromAddress)
  }

  async checkInstalled(): Promise<boolean> {
    for (let i = 1; i < 10 && !this.getTarget(); i += 1) {
      await new Promise((resolve) => setTimeout(resolve, 100 * i))
    }

    return Promise.resolve(this.getTarget() !== undefined)
  }
}
