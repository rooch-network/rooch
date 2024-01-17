// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

import { Bytes, IAuthorization, IAuthorizer } from '@roochnetwork/rooch-sdk'
import { BaseWallet } from '../types/wellet/baseWallet'
import { WalletAccount } from '../types'

// Sign transactions using a wallet
export class WalletAuth implements IAuthorizer {
  private readonly wallet: BaseWallet
  private readonly walletAccount: WalletAccount
  private readonly authInfo: string

  constructor(wallet: BaseWallet, walletAccount: WalletAccount, authInfo: string) {
    this.wallet = wallet
    this.walletAccount = walletAccount
    this.authInfo = authInfo
  }

  async auth(callData: Bytes): Promise<IAuthorization> {
    return {
      scheme: this.wallet.getScheme(),
      payload: await this.wallet.signMessage(callData, this.walletAccount, this.authInfo),
    }
  }
}
