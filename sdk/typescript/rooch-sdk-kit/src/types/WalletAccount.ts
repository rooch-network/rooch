// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

import { RoochMultiChainID } from '@roochnetwork/rooch-sdk'

import { MultiChainAddress } from './address'
import { SupportChain } from '../feature'

// import { address } from 'bitcoinjs-lib'

export class WalletAccount {
  private readonly address: string
  private readonly publicKey?: string
  private readonly compressedPublicKey?: string
  // TODO: add network info
  private readonly walletType: SupportChain

  public constructor(
    address: string,
    walletType: SupportChain,
    publicKey?: string,
    compressedPublicKey?: string,
  ) {
    this.address = address
    this.publicKey = publicKey
    this.walletType = walletType
    this.compressedPublicKey = compressedPublicKey
  }

  /**
   * Get account address
   */
  public getAddress(): string {
    return this.address
  }

  public toMultiChainAddress(): MultiChainAddress | null {
    if (this.walletType !== SupportChain.ETH) {
      return new MultiChainAddress(RoochMultiChainID.Bitcoin, this.address)
    }

    return null
  }

  public getInfo() {
    return {
      address: this.address,
      publicKey: this.publicKey,
      compressedPublicKey: this.compressedPublicKey,
    }
  }
}
