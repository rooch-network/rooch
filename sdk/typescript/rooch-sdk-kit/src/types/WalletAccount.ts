// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

import { IAccount, IAuthorizer, RoochClient, RoochMultiChainID } from '@roochnetwork/rooch-sdk'

import { MultiChainAddress } from './address'
import { SupportChain } from '../feature'
import { chain2MultiChainID } from '../utils/chain2MultiChainID'

export class WalletAccount implements IAccount {
  public readonly chain: SupportChain
  public readonly client: RoochClient
  public readonly address: string
  public readonly authorization: IAuthorizer
  public readonly publicKey?: string
  public readonly compressedPublicKey?: string

  private roochAddress?: string

  public constructor(
    client: RoochClient,
    chain: SupportChain,
    address: string,
    authorization: IAuthorizer,
    publicKey?: string,
    compressedPublicKey?: string,
  ) {
    this.chain = chain
    this.client = client
    this.address = address
    this.authorization = authorization
    this.publicKey = publicKey
    this.compressedPublicKey = compressedPublicKey
  }

  toJSON(): any {
    return {}
  }

  public toMultiChainAddress(): MultiChainAddress | null {
    if (this.chain !== SupportChain.ETH) {
      return new MultiChainAddress(RoochMultiChainID.Bitcoin, this.address)
    }

    return null
  }

  getAddress(): string {
    return this.address
  }

  async resoleRoochAddress(): Promise<string> {
    if (!this.roochAddress) {
      this.roochAddress = await this.client.resoleRoochAddress({
        address: this.address,
        multiChainID: chain2MultiChainID(this.chain),
      })
    }
    return this.roochAddress
  }

  getAuthorizer(): IAuthorizer {
    return this.authorization
  }
}
