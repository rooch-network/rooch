// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

import { RoochMultiChainID } from '@roochnetwork/rooch-sdk'

import { MultiChainAddress } from './address'

// import { address } from 'bitcoinjs-lib'

export class WalletAccount {
  private readonly address: string
  private readonly publicKey?: string
  private readonly compressedPublicKey?: string

  public constructor(address: string, publicKey?: string, compressedPublicKey?: string) {
    this.address = address
    this.publicKey = publicKey
    this.compressedPublicKey = compressedPublicKey
  }

  /**
   * Get account address
   */
  public getAddress(): string {
    return this.address
  }

  public toMultiChainAddress(): MultiChainAddress {
    return new MultiChainAddress(RoochMultiChainID.Bitcoin, this.address)
  }

  public getInfo() {
    return {
      address: this.address,
      publicKey: this.publicKey,
      compressedPublicKey: this.compressedPublicKey,
    }
  }
}
