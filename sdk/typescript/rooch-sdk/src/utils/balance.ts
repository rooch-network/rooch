// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

export function fixedBalance(balance: string | number, decimals: string | number) {
  balance = typeof balance === 'number' ? balance : Number(balance)
  decimals = typeof decimals === 'number' ? decimals : Number(decimals)

  return balance / Math.pow(10, decimals)
}
