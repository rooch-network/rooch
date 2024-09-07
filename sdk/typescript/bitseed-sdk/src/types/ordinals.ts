// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0
export type OutPoint = {
  txid: string
  vout: number
}

export type SatPoint = {
  outpoint: OutPoint
  offset: number
}

export type FeeRate = number
export type Amount = number
