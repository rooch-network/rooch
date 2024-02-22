// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

import { sha3_256 } from '@noble/hashes/sha3'
import { WalletAccount } from '../WalletAccount'
import { SerializedSignature } from '@roochnetwork/rooch-sdk'
import { Buffer } from 'buffer'

export const RoochSignPrefix = 'Rooch tx hash:\n'

export abstract class BaseWallet {
  account?: WalletAccount

  abstract connect(): Promise<WalletAccount[]>
  abstract sign(msg: string): Promise<string>
  abstract switchNetwork(): void
  abstract getNetwork(): string
  abstract getSupportNetworks(): string[]
  abstract onAccountsChanged(callback: (account: Array<string>) => void): void
  abstract removeAccountsChanged(callback: (account: Array<string>) => void): void
  abstract onNetworkChanged(callback: (network: string) => void): void
  abstract removeNetworkChanged(callback: (network: string) => void): void
  abstract normalize_recovery_id(recoveryID: number): number
  abstract getTarget(): any
  abstract getScheme(): number
  protected abstract toSerializedSignature(
    msg: string,
    signature: string,
    signatureInfo: string,
  ): SerializedSignature

  async signMessage(msg: Uint8Array, msgInfo?: any) {
    const digest = sha3_256(msg)
    return await this.signMessageWithHashed(digest, msgInfo)
  }

  async signMessageWithHashed(msgHash: Uint8Array, msgInfo: any) {
    let msgHex = Buffer.from(msgHash).toString('hex')

    if (msgInfo.charAt(msgInfo.length - 1) !== '\n') {
      msgInfo += '\n'
    }

    msgInfo = msgInfo + RoochSignPrefix
    let fullMsg = msgInfo + msgHex

    const sign = await this.sign(fullMsg)

    return this.toSerializedSignature(msgHex, sign, msgInfo)
  }

  async checkInstalled(): Promise<boolean> {
    for (let i = 1; i < 10 && !this.getTarget(); i += 1) {
      await new Promise((resolve) => setTimeout(resolve, 100 * i))
    }

    return Promise.resolve(this.getTarget() !== undefined)
  }
}
