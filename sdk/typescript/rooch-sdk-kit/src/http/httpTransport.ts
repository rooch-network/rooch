// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

import {
  RoochHTTPTransport,
  RoochTransportRequestOptions,
  RoochHTTPTransportOptions,
} from '@roochnetwork/rooch-sdk'

export class HTTPTransport extends RoochHTTPTransport {
  private readonly requestErrorCallback: (code: number, msg: string) => void

  constructor(
    options: RoochHTTPTransportOptions,
    requestErrorCallback: (code: number, msg: string) => void,
  ) {
    super(options)
    this.requestErrorCallback = requestErrorCallback
  }

  async request<T>(input: RoochTransportRequestOptions): Promise<T> {
    let result: T
    try {
      result = await super.request(input)
      return result
    } catch (e: any) {
      if ('code' in e) {
        this.requestErrorCallback(e.code, e.message)
      }
      throw e
    }
  }
}
