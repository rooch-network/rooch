// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

import { Address, BitcoinAddress, Bytes } from '@roochnetwork/rooch-sdk'
import { BitcoinWallet } from '@/wellet/bitcoin'

const UNISAT_SUPPORT_NETWORKS = ['livenet', 'testnet']

export class UniSatWallet extends BitcoinWallet {
  getName(): string {
    return 'unisat'
  }

  getTarget(): any {
    return (window as any).unisat
  }

  async connect(): Promise<Address[]> {
    let accounts: string[] = await this.getTarget().getAccounts()

    if (!accounts || accounts.length === 0) {
      await this.getTarget().requestAccounts()
      return this.connect()
    }

    return accounts.map((item) => new BitcoinAddress(item))
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

  onAccountsChanged(callback: (account: string[]) => void): void {
    this.getTarget().on('accountsChanged', callback)
  }

  removeAccountsChanged(callback: (account: string[]) => void): void {
    this.getTarget().removeListener('accountsChanged', callback)
  }

  onNetworkChanged(callback: (network: string) => void): void {
    this.getTarget().on('networkChanged', callback)
  }

  removeNetworkChanged(callback: (network: string) => void): void {
    this.getTarget().removeListener('networkChanged', callback)
  }

  async sign(msg: Bytes): Promise<Bytes> {
    return await this.getTarget().signMessage(msg)
  }
}
