// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

import { WalletAccount } from '../WalletAccount'
import { BitcoinWallet } from './bitcoinWallet'
import { SupportChain } from '../../feature'

const UNISAT_SUPPORT_NETWORKS = ['livenet', 'testnet']

export class UniSatWallet extends BitcoinWallet {
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
    let publicKey = await this.getTarget().getPublicKey()

    const walletAccounts = accounts.map((value, index) => {
      if (index === 0) {
        // unisat only supports the current account to get publicKey
        return new WalletAccount(value, '', SupportChain.BITCOIN, publicKey)
      } else {
        return new WalletAccount(value, '', SupportChain.BITCOIN)
      }
    })

    this.account = walletAccounts[0]

    return walletAccounts
  }

  switchNetwork(): void {
    this.getTarget().switchNetwork()
  }
  getNetwork(): string {
    return this.getTarget().getNetwork()
  }
  getSupportNetworks(): string[] {
    return UNISAT_SUPPORT_NETWORKS
  }
  onAccountsChanged(callback: (account: WalletAccount[]) => void): void {
    this.getTarget().on('accountsChanged', callback)
  }
  removeAccountsChanged(callback: (account: WalletAccount[]) => void): void {
    this.getTarget().removeListener('accountsChanged', callback)
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
