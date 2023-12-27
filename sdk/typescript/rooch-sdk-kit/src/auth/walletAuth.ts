// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

import { Bytes, IAuthorization, IAuthorizer } from '@roochnetwork/rooch-sdk'
import { BaseWallet } from '../types/wellet/baseWallet'

// Sign transactions using a wallet
export class WalletAuth implements IAuthorizer {
  private readonly wallet: BaseWallet

  constructor(wallet: BaseWallet) {
    this.wallet = wallet
  }

  async auth(callData: Bytes): Promise<IAuthorization> {
    return {
      scheme: this.wallet.getScheme(),
      payload: await this.wallet.signMessage(callData),
    }
  }
}
