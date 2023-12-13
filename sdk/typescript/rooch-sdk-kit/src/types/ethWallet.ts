// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

import { ChainInfo } from '@roochnetwork/rooch-sdk'

export class ETHWallet {
  async addChain(chain: ChainInfo) {
    return window.ethereum
      ?.request({
        method: 'wallet_addEthereumChain',
        params: [chain],
      })
      .then((v) => {
        console.log(v)
      })
  }

  async switchChain(chain: ChainInfo, { defaultAdd = true }: { defaultAdd?: boolean } = {}) {
    try {
      await window.ethereum?.request({
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
