// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

import { Wallet } from '@roochnetwork/rooch-sdk-kit'

export class Ton extends Wallet {
  connect(): Promise<[]> {
    return Promise.resolve([])
  }

  getBalance(): Promise<{ confirmed: number; unconfirmed: number; total: string }> {
    return Promise.resolve({ confirmed: 0, total: '', unconfirmed: 0 })
  }

  getBitcoinAddress(): BitcoinAddress {
    return undefined
  }

  getChain(): SupportChain {
    return undefined
  }

  getDescription(): string {
    return ''
  }

  getIcon(theme?: "dark" | "light"): string {
    return ''
  }

  getInstallUrl(): string {
    return ''
  }

  getKeyScheme(): SignatureScheme {
    return undefined
  }

  getName(): string {
    return ''
  }

  getNetwork(): WalletNetworkType {
    return undefined
  }

  getPublicKey(): PublicKey<Address> {
    return undefined
  }

  getRoochAddress(): RoochAddress {
    return undefined
  }

  getSupportNetworks(): WalletNetworkType[] {
    return []
  }

  getTarget(): any {
  }

  protected normalize_recovery_id(recoveryID: number): number {
    return 0
  }

  onAccountsChanged(callback: (accounts: Array<string>) => void): void {
  }

  onNetworkChanged(callback: (network: string) => void): void {
  }

  removeAccountsChanged(callback: (accounts: Array<string>) => void): void {
  }

  removeNetworkChanged(callback: (network: string) => void): void {
  }

  sendBtc(input: { toAddress: string; satoshis: number; options?: { feeRate: number } }): Promise<string> {
    return Promise.resolve('')
  }

  sign(msg: Bytes): Promise<Bytes>
  sign(input: Bytes): Promise<Bytes>
  sign(msg: Bytes): Promise<Bytes> {
    return Promise.resolve(undefined)
  }

  signTransaction(input: Transaction): Promise<Authenticator> {
    return Promise.resolve(undefined)
  }

  switchAccount(address: string): void {
  }

  switchNetwork(network: WalletNetworkType): void {
  }
}
