// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

import { ChainInfo, SerializedSignature } from '@roochnetwork/rooch-sdk'
import { BaseWallet } from './baseWallet'
import { AuthenticatorPayload } from '../AuthenticatorPayload'
import { Buffer } from 'buffer'
import { WalletAccount } from '../WalletAccount'

// TODO: eip6963 Discovered Wallets
// interface ETHWalletInfo {
//   uuid: string
//   name: string
//   icon: string
//   rdns: string
// }

// const DefaultInfo: ETHWalletInfo = {
//   uuid: 'd36e641f-9cd0-415d-9edb-bc7dc9f8f399',
//   name: 'MetaMask',
//   icon: '',
//   rdns: 'io.metamask',
// }
const ETH_MAGIC_SIGN_PREFIX = '\u0019Ethereum Signed Message:\n'

export abstract class ETHWallet extends BaseWallet {
  async addChain(chain: ChainInfo) {
    await this.getTarget().request({
      method: 'wallet_addEthereumChain',
      params: [chain],
    })
  }

  async switchChain(chain: ChainInfo, { defaultAdd = true }: { defaultAdd?: boolean } = {}) {
    try {
      await this.getTarget().request({
        method: 'wallet_switchEthereumChain',
        params: [{ chainId: chain.chainId }],
      })
    } catch (e: any) {
      if (e.code === 4902 && defaultAdd) {
        await this.addChain(chain)
      } else {
        // unknown error
        console.log(e)
        throw e
      }
    }
  }

  protected toSerializedSignature(
    msg: string,
    signature: string,
    signatureInfo: string,
    walletAccount: WalletAccount,
  ): SerializedSignature {
    let signBuffer = Buffer.from(signature.slice(2), 'hex')

    const normalizeSignBuffer = Buffer.concat([
      signBuffer.subarray(0, signBuffer.length - 1),
      Buffer.from([this.normalize_recovery_id(signBuffer[signBuffer.length - 1])]),
    ])

    let signatureInfoBytes = Buffer.from(signatureInfo)

    const prefix_buf = Buffer.from(
      `${ETH_MAGIC_SIGN_PREFIX}${msg.length + signatureInfoBytes.length}`,
      'utf-8',
    )

    let authPayload = new AuthenticatorPayload(
      Array.from(normalizeSignBuffer),
      Array.from(prefix_buf),
      Array.from(signatureInfoBytes),
      [],
      [],
      Array.from(Buffer.from(walletAccount.getAddress().substring(2), 'hex')),
    )

    console.log(authPayload)

    return authPayload.toBytes()
  }

  normalize_recovery_id(recoveryID: number, chainId?: number): number {
    if (recoveryID === 0 || recoveryID === 1) return recoveryID

    if (chainId === undefined) {
      return recoveryID - 27
    }

    return recoveryID - (chainId * 2 + 35)
  }
}
