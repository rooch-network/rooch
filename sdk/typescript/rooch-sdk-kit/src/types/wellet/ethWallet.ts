// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

import { ChainInfo } from '@roochnetwork/rooch-sdk'
import { BaseWallet } from './baseWallet'

// TODO: https://metamask.github.io/test-dapp/#personalSign or eth_sign
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
}
