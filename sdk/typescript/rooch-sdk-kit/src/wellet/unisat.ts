// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

import { BitcoinAddress, Bytes, ThirdPartyAddress, str, bytes } from '@roochnetwork/rooch-sdk'

import { BitcoinWallet } from '../wellet/index.js'
import { sha256 } from '@roochnetwork/rooch-sdk'

const UNISAT_SUPPORT_NETWORKS = ['livenet', 'testnet']

export class UniSatWallet extends BitcoinWallet {
  getName(): string {
    return 'unisat'
  }

  getTarget(): any {
    return (window as any).unisat
  }

  async connect(): Promise<ThirdPartyAddress[]> {
    let addresses: string[] = await this.getTarget().getAccounts()

    if (!addresses || addresses.length === 0) {
      await this.getTarget().requestAccounts()
      return this.connect()
    }

    let publicKey = await this.getTarget().getPublicKey()

    this.address = addresses.map((item) => new BitcoinAddress(item))
    this.currentAddress = this.address[0]
    this.publicKey = publicKey

    return this.address
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
    const msgStr = str('utf8', msg)
    console.log('msg hash', sha256('\u0018Bitcoin Signed Message:\n' + msgStr))
    const sign = await this.getTarget().signMessage(msgStr)

    return bytes('base64', sign).subarray(1) // remove recover id
  }
}
