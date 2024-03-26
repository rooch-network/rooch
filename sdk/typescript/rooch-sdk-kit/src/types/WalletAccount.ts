// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

import { RoochMultiChainID } from '@roochnetwork/rooch-sdk'

import { MultiChainAddress } from './address'
import { SupportChain } from '../feature'

export class WalletAccount {
  public readonly address: string
  public readonly publicKey?: string
  public readonly compressedPublicKey?: string
  // TODO: add network info
  public readonly walletType: SupportChain
  public readonly roochAddress: string

  public constructor(
    address: string,
    roochAddress: string,
    walletType: SupportChain,
    publicKey?: string,
    compressedPublicKey?: string,
  ) {
    this.address = address
    this.roochAddress = roochAddress
    this.publicKey = publicKey
    this.walletType = walletType
    this.compressedPublicKey = compressedPublicKey
  }

  public toMultiChainAddress(): MultiChainAddress | null {
    if (this.walletType !== SupportChain.ETH) {
      return new MultiChainAddress(RoochMultiChainID.Bitcoin, this.address)
    }

    return null
  }
}
