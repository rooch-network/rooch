// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

export const USER_REJECTED = 4001

export const NOT_SUPPORT_ADDRESS = 99991
export const AMOUNT_INVALID = 99999

export class WalletError extends Error {
  code: number

  constructor(code: number, msg: string) {
    super(msg)
    this.code = code
  }
}
