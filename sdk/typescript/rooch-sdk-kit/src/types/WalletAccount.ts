// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

import { IAccount, IAuthorizer, RoochClient, RoochMultiChainID } from '@roochnetwork/rooch-sdk'

import { MultiChainAddress } from './address'
import { SupportChain } from '../feature'
import { chain2MultiChainID } from '../utils/chain2MultiChainID'

export class WalletAccount implements IAccount {
  public readonly address: string
  public readonly publicKey?: string
  public readonly compressedPublicKey?: string

  private chain: SupportChain
  private authorization: IAuthorizer
  private client?: RoochClient
  private roochAddress?: string

  public constructor(
    chain: SupportChain,
    authorization: IAuthorizer,
    address: string,
    client?: RoochClient,
    publicKey?: string,
    compressedPublicKey?: string,
  ) {
    this.chain = chain
    this.client = client
    this.authorization = authorization
    this.address = address
    this.publicKey = publicKey
    this.compressedPublicKey = compressedPublicKey
  }

  public toMultiChainAddress(): MultiChainAddress | null {
    if (this.chain !== SupportChain.ETH) {
      return new MultiChainAddress(RoochMultiChainID.Bitcoin, this.address)
    }

    return null
  }

  getAddress(): string | undefined {
    return this.address
  }

  async getRoochAddress(): Promise<string> {
    if (!this.client) {
      throw new Error()
    }

    if (!this.roochAddress) {
      this.roochAddress = await this.client?.resoleRoochAddress({
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
