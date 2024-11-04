// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

export type Proof = {
  name: string
  proof: {
    timestamp: number
    domain: {
      lengthBytes: number
      value: string
    }
    signature: string
    payload: string
  }
  stateInit: string
}
