// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0
import debug from 'debug'
import { sleep } from './time'
import rooch_sdk from '@roochnetwork/rooch-sdk'

const log = debug('bitseed:e2e:http_debug_transport')
const { RoochHTTPTransport } = rooch_sdk

export class HTTPDebugTransport extends RoochHTTPTransport {
  private debug: boolean

  constructor(options, debug) {
    super(options)

    this.debug = debug
  }

  async request<T>(input): Promise<T> {
    let result: T

    try {
      if (this.debug) {
        log('rooch http request start:', input)
      }

      result = await super.request(input)

      if (this.debug) {
        log('rooch http request result:', input, result)

        if (input.method === 'btc_queryUTXOs') {
          let resp = result as any

          if (resp.data.length === 0) {
            log('rooch btc_queryUTXOs result empty, sleep 3s ...')

            await sleep(3000)
          }
        }
      }

      return result
    } catch (e: any) {
      if (this.debug) {
        log('rooch http request error:', input, e)
      }

      throw e
    }
  }
}
