// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

import { BitcoinAddress, Bytes, ThirdPartyAddress, str, bytes } from '@roochnetwork/rooch-sdk'
import { BitcoinWallet } from '../wellet/index.js'

export class XVerseWallet extends BitcoinWallet {
  // private callback : Map<string, () => void>

  getName(): string {
    return 'Xverse'
  }

  getIcon(_?: 'dark' | 'light'): string {
    // if (theme === 'dark') {
    //   return ''
    // }

    return 'data:image/svg+xml;base64,PHN2ZyB4bWxucz0iaHR0cDovL3d3dy53My5vcmcvMjAwMC9zdmciIHdpZHRoPSI2MDAiIGhlaWdodD0iNjAwIj48ZyBmaWxsPSJub25lIiBmaWxsLXJ1bGU9ImV2ZW5vZGQiPjxwYXRoIGZpbGw9IiMxNzE3MTciIGQ9Ik0wIDBoNjAwdjYwMEgweiIvPjxwYXRoIGZpbGw9IiNGRkYiIGZpbGwtcnVsZT0ibm9uemVybyIgZD0iTTQ0MCA0MzUuNHYtNTFjMC0yLS44LTMuOS0yLjItNS4zTDIyMCAxNjIuMmE3LjYgNy42IDAgMCAwLTUuNC0yLjJoLTUxLjFjLTIuNSAwLTQuNiAyLTQuNiA0LjZ2NDcuM2MwIDIgLjggNCAyLjIgNS40bDc4LjIgNzcuOGE0LjYgNC42IDAgMCAxIDAgNi41bC03OSA3OC43Yy0xIC45LTEuNCAyLTEuNCAzLjJ2NTJjMCAyLjQgMiA0LjUgNC42IDQuNUgyNDljMi42IDAgNC42LTIgNC42LTQuNlY0MDVjMC0xLjIuNS0yLjQgMS40LTMuM2w0Mi40LTQyLjJhNC42IDQuNiAwIDAgMSA2LjQgMGw3OC43IDc4LjRhNy42IDcuNiAwIDAgMCA1LjQgMi4yaDQ3LjVjMi41IDAgNC42LTIgNC42LTQuNloiLz48cGF0aCBmaWxsPSIjRUU3QTMwIiBmaWxsLXJ1bGU9Im5vbnplcm8iIGQ9Ik0zMjUuNiAyMjcuMmg0Mi44YzIuNiAwIDQuNiAyLjEgNC42IDQuNnY0Mi42YzAgNCA1IDYuMSA4IDMuMmw1OC43LTU4LjVjLjgtLjggMS4zLTIgMS4zLTMuMnYtNTEuMmMwLTIuNi0yLTQuNi00LjYtNC42TDM4NCAxNjBjLTEuMiAwLTIuNC41LTMuMyAxLjNsLTU4LjQgNTguMWE0LjYgNC42IDAgMCAwIDMuMiA3LjhaIi8+PC9nPjwvc3ZnPg=='
  }

  getDescription(): string {
    return 'Xverse Wallet'
  }

  getInstallUrl(): string {
    return 'https://chrome.google.com/webstore/detail/xverse-wallet/idnnbdplmphpflfnlkomgpfbpcgelopg?hl=en-GB&authuser=1'
  }

  async sign(msg: Bytes): Promise<Bytes> {
    const msgStr = str('utf8', msg)
    const sign = await this.getTarget().signMessage(msgStr, {
      from: this.currentAddress?.toStr(),
    })
    return bytes('base64', sign).subarray(1)
  }

  getTarget(): any {
    return (window as any).XverseProviders?.BitcoinProvider
  }

  async connect(): Promise<ThirdPartyAddress[]> {
    let objs: any[] = await this.getTarget().getAccounts()

    this.address = objs.map((obj) => new BitcoinAddress(obj.address))
    this.currentAddress = this.address[0]
    this.publicKey = objs[0].publicKey

    return this.address
  }

  switchNetwork(): void {
    // this.getTarget().switchNetwork()
  }

  getNetwork(): string {
    return 'Mainnet'
    // return this.getTarget().getNetwork()
  }

  getSupportNetworks(): string[] {
    return ['Mainnet', 'Testnet', 'Signet']
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

  sendBtc(input: {
    toAddress: string
    satoshis: number
    options?: { feeRate: number }
  }): Promise<string> {
    return this.getTarget().sendBitcoin(input.toAddress, input.satoshis, input.options)
  }

  getBalance(): Promise<{ confirmed: number; unconfirmed: number; total: string }> {
    return this.getTarget().getBalance()
  }
}
