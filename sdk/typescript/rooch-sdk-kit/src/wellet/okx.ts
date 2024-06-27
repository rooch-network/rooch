// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

import { BitcoinAddress, Bytes, ThirdPartyAddress } from '@roochnetwork/rooch-sdk'
import { BitcoinWallet } from '../wellet/index.js'

export class OkxWallet extends BitcoinWallet {
  getName(): string {
    return 'okx'
  }

  async sign(msg: Bytes): Promise<Bytes> {
    return this.getTarget().signMessage(msg, {
      from: this.currentAddress?.toStr(),
    })
  }

  getTarget(): any {
    return (window as any).okxwallet.bitcoin
  }

  async connect(): Promise<ThirdPartyAddress[]> {
    const addr = await this.getTarget().connect()
    this.currentAddress = new BitcoinAddress(addr)
    this.address = [this.currentAddress]

    return this.address
  }

  switchNetwork(): void {
    this.getTarget().switchNetwork()
  }

  getNetwork(): string {
    return this.getTarget().getNetwork()
  }

  getSupportNetworks(): string[] {
    return ['livenet']
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
}
