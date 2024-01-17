// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

export enum SupportWallet {
  BITCOIN = 'BITCOIN',
  // Okx = 'Okx',
  ETH = 'ETH',
}

export const SupportWallets = [SupportWallet.BITCOIN, SupportWallet.ETH]

// TODO: multi wallet support
export class SupportNetWork {
  static get Bitcoin(): BitcoinWallets {
    return new BitcoinWallets()
  }

  static get ETH(): ETHSupportWallet {
    return new ETHSupportWallet()
  }
}

class BitcoinWallets {
  get Unisat() {
    console.log('Bitcoin with Unisat wallet support')
    return this
  }
}

class ETHSupportWallet {
  get Metamask() {
    console.log('Ethereum with Metamask support')
    return this
  }

  get OKX() {
    console.log('Ethereum with OKX support')
    return this
  }

  get Unisat() {
    console.log('Ethereum with Unisat support')
    return this
  }
}
