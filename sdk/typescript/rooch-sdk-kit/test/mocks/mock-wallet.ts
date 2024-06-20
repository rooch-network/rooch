// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

import { Keypair } from '@roochnetwork/rooch-sdk'
import { Wallet } from './wallet.js'

export class MockBitcoinWallet extends Wallet {
  kp: Keypair
  network: string[]

  constructor(kp: Keypair, network: string[]) {
    super()
    this.kp = kp
    this.network = network
  }

  connect(): Promise<any[]> {
    return Promise.resolve([])
  }

  getNetwork(): string {
    return ''
  }

  getSupportNetworks(): string[] {
    return []
  }

  onAccountsChanged(_: (accounts: Array<string>) => void): void {}

  onNetworkChanged(_: (network: string) => void): void {}

  removeAccountsChanged(_: (accounts: Array<string>) => void): void {}

  removeNetworkChanged(_: (network: string) => void): void {}

  async sign(msg: any): Promise<any> {
    return this.kp.sign(msg)
  }

  switchAccount(_: string): void {}

  switchNetwork(_: string): void {}
}
