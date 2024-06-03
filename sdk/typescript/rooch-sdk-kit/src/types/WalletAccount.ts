// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

import { IAccount, IAuthorizer, RoochClient } from '@roochnetwork/rooch-sdk'

import { SupportChain } from '../feature'
import { chain2MultiChainID } from '../utils/chain2MultiChainID'
import {bech32m} from 'bech32';
import {Buffer} from 'buffer';

export class WalletAccount implements IAccount {
  public readonly chain: SupportChain
  public readonly client: RoochClient
  public readonly address: string
  public readonly authorization: IAuthorizer
  public readonly publicKey?: string
  public readonly compressedPublicKey?: string

  private roochHexAddress?: string
  private roochBech32Address?: string

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

  getAddress(): string {
    return this.address
  }

  getRoochAddress(): string {
    return this.roochHexAddress!
  }
  getBech32RoochAddress(): string {
    if (!this.roochBech32Address) {
      let rad = this.roochHexAddress!.substring(2)
      return bech32m.encode('rooch', bech32m.toWords(Buffer.from(rad,'hex')))
    }
    return this.roochBech32Address
  }

  async resoleRoochAddress(): Promise<string> {
    if (!this.roochHexAddress) {
      this.roochHexAddress = await this.client.resoleRoochAddress({
        address: this.address,
        multiChainID: chain2MultiChainID(this.chain),
      })
    }
    return this.roochHexAddress
  }

  getAuthorizer(): IAuthorizer {
    return this.authorization
  }
}
