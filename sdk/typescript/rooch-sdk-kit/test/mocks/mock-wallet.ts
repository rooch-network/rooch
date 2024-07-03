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
import { Mock } from 'vitest'

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

  getNetwork(): string {
    return 'testnet'
  }

  getPublicKey(): PublicKey<Address> {
    return this.kp.getPublicKey()
  }

  getRoochAddress(): RoochAddress {
    return this.kp.getRoochAddress()
  }

  getSupportNetworks(): string[] {
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

  switchNetwork(_: string): void {}
}
