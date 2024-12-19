// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

import { BitcoinAddress, Bytes, ThirdPartyAddress, str, bytes } from '@roochnetwork/rooch-sdk'
import { BitcoinWallet } from '../wellet/index.js'
import { All_NETWORK, WalletNetworkType } from './types.js'

export class OnekeyWallet extends BitcoinWallet {
  getName(): string {
    return 'OneKey'
  }

  getIcon(theme?: 'dark' | 'light'): string {
    if (theme === 'dark') {
      return 'data:image/svg+xml;base64,PHN2ZyB3aWR0aD0iMTQ0IiBoZWlnaHQ9IjE0NCIgdmlld0JveD0iMCAwIDE0NCAxNDQiIGZpbGw9Im5vbmUiIHhtbG5zPSJodHRwOi8vd3d3LnczLm9yZy8yMDAwL3N2ZyI+CjxwYXRoIGZpbGwtcnVsZT0iZXZlbm9kZCIgY2xpcC1ydWxlPSJldmVub2RkIiBkPSJNNzIgMTQ0QzEyMS43MDYgMTQ0IDE0NCAxMjEuNzA2IDE0NCA3MkMxNDQgMjIuMjk0NCAxMjEuNzA2IDAgNzIgMEMyMi4yOTQ0IDAgMCAyMi4yOTQ0IDAgNzJDMCAxMjEuNzA2IDIyLjI5NDQgMTQ0IDcyIDE0NFpNNTguNDc1MyAzMC41MzA1SDc4LjUwNTNWNjMuNTM4MUg2Ni4wODY1VjQxLjE1NjJINTQuOTYxM0w1OC40NzUzIDMwLjUzMDVaTTcyLjAwMDQgMTEzLjQ2OUM4NC42MTY0IDExMy40NjkgOTQuODQzNyAxMDMuMjQyIDk0Ljg0MzcgOTAuNjI2MUM5NC44NDM3IDc4LjAxMDEgODQuNjE2NCA2Ny43ODI4IDcyLjAwMDQgNjcuNzgyOEM1OS4zODQ0IDY3Ljc4MjggNDkuMTU3MSA3OC4wMTAxIDQ5LjE1NzEgOTAuNjI2MUM0OS4xNTcxIDEwMy4yNDIgNTkuMzg0NCAxMTMuNDY5IDcyLjAwMDQgMTEzLjQ2OVpNNzIuMDAwNCAxMDMuMDk5Qzc4Ljg4ODkgMTAzLjA5OSA4NC40NzMxIDk3LjUxNDUgODQuNDczMSA5MC42MjZDODQuNDczMSA4My43Mzc1IDc4Ljg4ODkgNzguMTUzMyA3Mi4wMDA0IDc4LjE1MzNDNjUuMTExOSA3OC4xNTMzIDU5LjUyNzYgODMuNzM3NSA1OS41Mjc2IDkwLjYyNkM1OS41Mjc2IDk3LjUxNDUgNjUuMTExOSAxMDMuMDk5IDcyLjAwMDQgMTAzLjA5OVoiIGZpbGw9ImJsYWNrIiBzdHlsZT0iZmlsbDpibGFjaztmaWxsLW9wYWNpdHk6MTsiLz4KPC9zdmc+Cg=='
    }

    return 'data:image/svg+xml;base64,PHN2ZyB3aWR0aD0iMTQ0IiBoZWlnaHQ9IjE0NCIgdmlld0JveD0iMCAwIDE0NCAxNDQiIGZpbGw9Im5vbmUiIHhtbG5zPSJodHRwOi8vd3d3LnczLm9yZy8yMDAwL3N2ZyI+CjxwYXRoIGZpbGwtcnVsZT0iZXZlbm9kZCIgY2xpcC1ydWxlPSJldmVub2RkIiBkPSJNNzIgMTQ0QzEyMS43MDYgMTQ0IDE0NCAxMjEuNzA2IDE0NCA3MkMxNDQgMjIuMjk0NCAxMjEuNzA2IDAgNzIgMEMyMi4yOTQ0IDAgMCAyMi4yOTQ0IDAgNzJDMCAxMjEuNzA2IDIyLjI5NDQgMTQ0IDcyIDE0NFpNNTguNDc1MyAzMC41MzA1SDc4LjUwNTNWNjMuNTM4MUg2Ni4wODY1VjQxLjE1NjJINTQuOTYxM0w1OC40NzUzIDMwLjUzMDVaTTcyLjAwMDQgMTEzLjQ2OUM4NC42MTY0IDExMy40NjkgOTQuODQzNyAxMDMuMjQyIDk0Ljg0MzcgOTAuNjI2MUM5NC44NDM3IDc4LjAxMDEgODQuNjE2NCA2Ny43ODI4IDcyLjAwMDQgNjcuNzgyOEM1OS4zODQ0IDY3Ljc4MjggNDkuMTU3MSA3OC4wMTAxIDQ5LjE1NzEgOTAuNjI2MUM0OS4xNTcxIDEwMy4yNDIgNTkuMzg0NCAxMTMuNDY5IDcyLjAwMDQgMTEzLjQ2OVpNNzIuMDAwNCAxMDMuMDk5Qzc4Ljg4ODkgMTAzLjA5OSA4NC40NzMxIDk3LjUxNDUgODQuNDczMSA5MC42MjZDODQuNDczMSA4My43Mzc1IDc4Ljg4ODkgNzguMTUzMyA3Mi4wMDA0IDc4LjE1MzNDNjUuMTExOSA3OC4xNTMzIDU5LjUyNzYgODMuNzM3NSA1OS41Mjc2IDkwLjYyNkM1OS41Mjc2IDk3LjUxNDUgNjUuMTExOSAxMDMuMDk5IDcyLjAwMDQgMTAzLjA5OVoiIGZpbGw9IiM0NEQ2MkMiIHN0eWxlPSJmaWxsOiM0NEQ2MkM7ZmlsbDpjb2xvcihkaXNwbGF5LXAzIDAuMjY2NyAwLjgzOTIgMC4xNzI1KTtmaWxsLW9wYWNpdHk6MTsiLz4KPC9zdmc+Cg=='
  }

  getDescription(): string {
    return 'OneKey Wallet'
  }

  getInstallUrl(): string {
    return 'https://chromewebstore.google.com/detail/onekey/jnmbobjmhlngoefaiojfljckilhhlhcj'
  }

  async sign(msg: Bytes): Promise<Bytes> {
    const msgStr = str('utf8', msg)
    const sign = await this.getTarget().signMessage(msgStr)
    return bytes('base64', sign).subarray(1)
  }

  getTarget(): any {
    return (window as any).$onekey?.btc
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

  // TODO: onekey provider switch api, But it doesn't work.
  switchNetwork(_: WalletNetworkType): Promise<void> {
    throw Error('onekey not support switch network!')
  }

  getNetwork(): Promise<WalletNetworkType> {
    return this.getTarget().getNetwork()
  }

  getSupportNetworks(): WalletNetworkType[] {
    return All_NETWORK
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
