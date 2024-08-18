// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0
export class BitseedSDKError extends Error {
  constructor(message: string) {
    super(message)
    this.name = 'BitseedSDKError'
  }
}
