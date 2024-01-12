// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

import { sha3_256 } from '@noble/hashes/sha3'
import { WalletAccount } from '../WalletAccount'
import { SerializedSignature } from '@roochnetwork/rooch-sdk'
import { Buffer } from 'buffer'

export const RoochSignPrefix = 'Rooch tx hash:\n'

export abstract class BaseWallet {
  protected abstract sign(msg: string, fromAddress: string): Promise<string>
  protected abstract toSerializedSignature(
    msg: string,
    signature: string,
    signatureInfo: string,
    walletAccount: WalletAccount,
  ): SerializedSignature

  abstract normalize_recovery_id(recoveryID: number): number
  abstract getTarget(): any
  abstract getScheme(): number

  abstract connect(): Promise<WalletAccount[]>

  async signMessage(msg: Uint8Array, walletAccount: WalletAccount, msgInfo?: any) {
    const digest = sha3_256(msg)
    return await this.signMessageWithHashed(digest, walletAccount, msgInfo)
  }

  async signMessageWithHashed(msgHash: Uint8Array, walletAccount: WalletAccount, msgInfo: any) {
    let msgHex = Buffer.from(msgHash).toString('hex')

    if (msgInfo.charAt(msgInfo.length - 1) !== '\n') {
      msgInfo += '\n'
    }

    msgInfo = msgInfo + RoochSignPrefix
    let fullMsg = msgInfo + msgHex

    const sign = await this.sign(fullMsg, walletAccount.getAddress())

    return this.toSerializedSignature(msgHex, sign, msgInfo, walletAccount)
  }

  async checkInstalled(): Promise<boolean> {
    for (let i = 1; i < 10 && !this.getTarget(); i += 1) {
      await new Promise((resolve) => setTimeout(resolve, 100 * i))
    }

    return Promise.resolve(this.getTarget() !== undefined)
  }
}
