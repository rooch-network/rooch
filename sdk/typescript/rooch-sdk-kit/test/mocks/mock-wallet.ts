// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

import {
  Keypair,
  ThirdPartyAddress,
  BitcoinAddress,
  SignatureScheme,
  PublicKey,
  Address,
  RoochAddress,
  Bytes,
  Transaction,
  Authenticator,
  Secp256k1Keypair,
  BitcoinSignMessage,
} from '@roochnetwork/rooch-sdk'
import { SupportChain } from '../../src/feature/index.js'
import { Wallet } from '../../src/wellet/wallet.js'
import { Mock, vi } from 'vitest'
import { WalletNetworkType } from '../../src/index.js'

export class MockBitcoinWallet extends Wallet {
  private kp: Keypair

  mocks: {
    connect: Mock
  }

  constructor() {
    super()
    this.kp = Secp256k1Keypair.generate()

    this.mocks = {
      connect: vi.fn().mockImplementation(() => [this.kp.getBitcoinAddress()]),
    }
  }

  connect(): Promise<ThirdPartyAddress[]> {
    return this.mocks.connect()
  }

  getBitcoinAddress(): BitcoinAddress {
    return this.kp.getBitcoinAddress()
  }

  getChain(): SupportChain {
    return 'bitcoin'
  }

  getKeyScheme(): SignatureScheme {
    return 'Secp256k1'
  }

  getName(): string {
    return 'mock'
  }

  getNetwork(): Promise<WalletNetworkType> {
    return Promise.resolve('testnet')
  }

  getPublicKey(): PublicKey<Address> {
    return this.kp.getPublicKey()
  }

  getRoochAddress(): RoochAddress {
    return this.kp.getRoochAddress()
  }

  getSupportNetworks(): WalletNetworkType[] {
    return ['testnet', 'livenet']
  }

  getTarget(): any {
    return this
  }

  protected normalize_recovery_id(recoveryID: number): number {
    return recoveryID
  }

  onAccountsChanged(_: (accounts: Array<string>) => void): void {}

  onNetworkChanged(_: (network: string) => void): void {}

  removeAccountsChanged(_: (accounts: Array<string>) => void): void {}

  removeNetworkChanged(_: (network: string) => void): void {}

  async sign(msg: Bytes): Promise<Bytes> {
    return await this.kp.sign(msg)
  }

  signTransaction(input: Transaction): Promise<Authenticator> {
    const message = new BitcoinSignMessage(input.hashData(), input.getInfo() || '')
    return Authenticator.bitcoin(message, this, 'raw')
  }

  switchAccount(_: string): void {}

  switchNetwork(_: string): Promise<void> {
    return Promise.resolve()
  }

  getDescription(): string {
    return ''
  }

  getIcon(theme?: 'dark' | 'light'): string {
    return theme ?? ''
  }

  getInstallUrl(): string {
    return ''
  }

  sendBtc(input: {
    toAddress: string
    satoshis: number
    options?: { feeRate: number }
  }): Promise<string> {
    throw new Error('Method not implemented.')
  }

  getBalance(): Promise<{ confirmed: number; unconfirmed: number; total: string }> {
    throw new Error('Method not implemented.')
  }
}
