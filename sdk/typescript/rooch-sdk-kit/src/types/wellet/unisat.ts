// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

import { WalletAccount } from '../WalletAccount'
import { BitcoinWallet } from './bitcoinWallet'
import { RoochClient } from '@roochnetwork/rooch-sdk'

const UNISAT_SUPPORT_NETWORKS = ['livenet', 'testnet']

export class UniSatWallet extends BitcoinWallet {
  constructor(client: RoochClient) {
    super(client)
    this.name = 'unisat'
  }

  private async analysisAccount(addresses: string[]): Promise<WalletAccount[]> {
    let publicKey = await this.getTarget().getPublicKey()

    const walletAccounts = addresses.map((address, index) => {
      if (index === 0) {
        // unisat only supports the current account to get publicKey
        return new WalletAccount(this.client, this.getChain(), address, this, publicKey)
      } else {
        return new WalletAccount(this.client, this.getChain(), address, this)
      }
    })

    this.account = walletAccounts[0]

    return walletAccounts
  }

  getTarget(): any {
    return (window as any).unisat
  }

  getScheme(): number {
    return 2
  }

  async connect(): Promise<WalletAccount[]> {
    let accounts: string[] = await this.getTarget().getAccounts()

    if (!accounts || accounts.length === 0) {
      await this.getTarget().requestAccounts()
      return this.connect()
    }

    return await this.analysisAccount(accounts)
  }

  switchNetwork(network: string): void {
    this.getTarget().switchNetwork(network)
  }
  getNetwork(): string {
    return this.getTarget().getNetwork()
  }
  getSupportNetworks(): string[] {
    return UNISAT_SUPPORT_NETWORKS
  }
  onAccountsChanged(callback: (account: WalletAccount[]) => void): void {
    if (!this.onAccountsChangedWrapper) {
      this.onAccountsChangedWrapper = async (addresses: string[]) => {
        callback(await this.analysisAccount(addresses))
      }
    }
    this.getTarget().on('accountsChanged', this.onAccountsChangedWrapper)
  }
  removeAccountsChanged(_: (account: WalletAccount[]) => void): void {
    this.getTarget().removeListener('accountsChanged', this.onAccountsChangedWrapper)
    this.onAccountsChangedWrapper = undefined
  }
  onNetworkChanged(callback: (network: string) => void): void {
    this.getTarget().on('networkChanged', callback)
  }
  removeNetworkChanged(callback: (network: string) => void): void {
    this.getTarget().removeListener('networkChanged', callback)
  }

  async sign(msg: string): Promise<string> {
    return await this.getTarget().signMessage(msg)
  }
}
