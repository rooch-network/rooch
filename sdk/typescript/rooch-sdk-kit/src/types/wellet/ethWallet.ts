// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

import { ChainInfo, SerializedSignature } from '@roochnetwork/rooch-sdk'
import { BaseWallet } from './baseWallet'
import { AuthenticatorPayload } from '../AuthenticatorPayload'
import { Buffer } from 'buffer'
import { SupportChain } from '../../feature'
import { WalletAccount } from '../../types'

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
  switchNetwork(_: string): void {
    throw new Error('Method not implemented.')
  }

  switchAccount(_: string) {
    throw new Error('Method not implemented.')
  }

  getNetwork(): string {
    throw new Error('Method not implemented.')
  }

  getSupportNetworks(): string[] {
    throw new Error('Method not implemented.')
  }

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

  onAccountsChanged(callback: (account: Array<WalletAccount>) => void) {
    this.getTarget().on('accountsChanged', callback)
  }

  removeAccountsChanged(callback: (account: Array<WalletAccount>) => void) {
    this.getTarget().removeListener('accountsChanged', callback)
  }

  onNetworkChanged(callback: (network: string) => void) {
    this.getTarget().on('chainChanged', callback)
  }

  removeNetworkChanged(callback: (network: string) => void) {
    this.getTarget().removeListener('chainChanged', callback)
  }

  normalize_recovery_id(recoveryID: number, chainId?: number): number {
    if (recoveryID === 0 || recoveryID === 1) return recoveryID

    if (chainId === undefined) {
      return recoveryID - 27
    }

    return recoveryID - (chainId * 2 + 35)
  }

  protected toSerializedSignature(
    msg: string,
    signature: string,
    signatureInfo: string,
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
      Array.from(Buffer.from(this.account!.address.substring(2), 'hex')),
    )

    return authPayload.toBytes()
  }

  getChain(): SupportChain {
    return SupportChain.ETH
  }
}
