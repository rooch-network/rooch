// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0
export type SFTContent = {
  content_type: string
  body: Uint8Array
}

export type SFTRecord = {
  op: string
  tick: string
  amount: number
  attributes?: {
    [key: string]: any
  }
  content?: SFTContent
}
