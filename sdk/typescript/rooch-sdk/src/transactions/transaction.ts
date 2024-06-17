// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

import { bcs } from '@/bcs'
import { Authenticator } from '@/crypto'
import { address, Bytes, u64 } from '@/types'

import { CallFunctionArgs, MoveAction, TransactionData } from './transactionData'

export class Transaction {
  private data: TransactionData | undefined
  private auth: Authenticator | undefined

  callFunction(input: CallFunctionArgs) {
    this.data = new TransactionData(MoveAction.newCallFunction(input))
  }

  setSender(input: address) {
    this.getData().sender = input
  }

  setAuth(input: Authenticator) {
    this.auth = input
  }

  setChainId(input: u64) {
    this.getData().chainId = input
  }

  setSeqNumber(input: u64) {
    this.getData().sequenceNumber = input
  }

  hashData(): Bytes {
    return this.getData().hash()
  }

  encode() {
    return bcs.RoochTransaction.serialize({
      data: this.data!.encode(),
      auth: this.auth!.encode(),
    })
  }

  private getData() {
    this.isValid()
    return this.data!
  }

  private isValid() {
    if (!this.data) {
      throw new Error('Transaction data is not initialized. Call action first.')
    }
  }
}
