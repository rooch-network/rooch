// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

import {
  RoochHTTPTransport,
  RoochTransportRequestOptions,
  RoochHTTPTransportOptions,
  ErrorValidateInvalidAccountAuthKey,
  ErrorValidateSessionIsExpired,
} from '@roochnetwork/rooch-sdk'

type SessionExpiredCallbackType = () => void

export class HTTPTransport extends RoochHTTPTransport {
  private readonly sessionExpiredCallback: SessionExpiredCallbackType

  constructor(
    options: RoochHTTPTransportOptions,
    sessionExpiredCallback: SessionExpiredCallbackType,
  ) {
    super(options)
    this.sessionExpiredCallback = sessionExpiredCallback
  }

  async request<T>(input: RoochTransportRequestOptions): Promise<T> {
    let result: T
    try {
      result = await super.request(input)
      return result
    } catch (e: any) {
      if (
        'code' in e &&
        (e.code === ErrorValidateInvalidAccountAuthKey || e.code === ErrorValidateSessionIsExpired)
      ) {
        this.sessionExpiredCallback()
      }
      throw e
    }
  }
}
