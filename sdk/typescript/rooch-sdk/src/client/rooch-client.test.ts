// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

import { describe, it, expect, vi } from 'vitest'
import fetchMock from 'fetch-mock'
import { DevChain } from '../constants/chain'
import { RoochClient } from './rooch-client'

describe('provider', () => {
  it('should create JsonRpcProvider ok ', async () => {
    const provider = new RoochClient()
    expect(provider).toBeDefined()
  })

  describe('#executeViewFunction', () => {
    it('should execute view function ok', async () => {
      const mockFetch = vi.fn().mockImplementation(() => {
        const mock = fetchMock.sandbox()

        const body = {
          jsonrpc: '2.0',
          result: [
            {
              value: {
                type_tag: 'u64',
                value: '0x0000000000000000',
              },
              decoded_value: '0',
            },
          ],
          id: '0',
        }

        mock.post('*', JSON.stringify(body))

        return mock
      })

      const provider = new RoochClient(DevChain, {
        fetcher: mockFetch,
      })
      expect(provider).toBeDefined()

      try {
        const fnId =
          '0x9fb8a2b8b84ea08804c5830c9fca7b6850cdb1a37aca58d33fea861ea1a515af::counter::value'
        const result = await provider.executeViewFunction(fnId)
        expect(result).toHaveLength(1)
      } catch (err: any) {
        expect(err).to.be.an('error')
      }
    })

    it('should execute view function error', async () => {
      const mockFetch = vi.fn().mockImplementation(() => {
        const mock = fetchMock.sandbox()
        const body = {
          jsonrpc: '2.0',
          error: {
            code: -32000,
            message: 'VMError with status LINKER_ERROR at location UNDEFINED',
          },
          id: '0',
        }

        mock.post('*', JSON.stringify(body))

        return mock
      })

      const provider = new RoochClient(DevChain, {
        fetcher: mockFetch,
      })
      expect(provider).toBeDefined()

      try {
        const fnId =
          '0xc74a6f9397b0eacd8fec9e95e9dc671da9ec69d3088688226070183edf84516::counter::value'
        await provider.executeViewFunction(fnId)
      } catch (err: any) {
        expect(err).to.be.an('error')
      }
    })

    describe('#getStates', () => {
      it('should get annotated statues ok', async () => {
        const mockFetch = vi.fn().mockImplementation(() => {
          const mock = fetchMock.sandbox()

          const body = {
            jsonrpc: '2.0',
            result: [
              {
                type_tag: 'u64',
                value: '0x0000000000000000',
                decoded_value: '0',
              },
            ],
            id: '0',
          }

          mock.post('*', JSON.stringify(body))

          return mock
        })

        const provider = new RoochClient(DevChain, {
          fetcher: mockFetch,
        })
        expect(provider).toBeDefined()

        try {
          const assetsPath = '/object::0x1'
          const result = await provider.getStates(assetsPath)

          expect(result).toHaveLength(1)
        } catch (err: any) {
          expect(err).to.be.an('error')
        }
      })
    })

    describe('#listStates', () => {
      it('should list annotated states ok', async () => {
        const mockFetch = vi.fn().mockImplementation(() => {
          const mock = fetchMock.sandbox()

          const body = {
            "jsonrpc": "2.0",
              "result": {
            "data": [
              {
                "key_state": {
                  "key": "0x65303030303030303030303030303030303030303030303030303030303030303030303030303030303030303030303030303030303030303030303030303030333a3a6163636f756e745f636f696e5f73746f72653a3a4175746f416363657074436f696e73",
                  "key_type": "0x1::ascii::String",
                  "decoded_key": "0000000000000000000000000000000000000000000000000000000000000003::account_coin_store::AutoAcceptCoins"
                },
                "state": {
                  "value": "0x0db9d01a4f72e3708665fde27fc0da6cf353b0a9b0a2f3a2c2597f3e949e62d8",
                  "value_type": "0x3::account_coin_store::AutoAcceptCoins",
                  "decoded_value": {
                    "abilities": 8,
                    "type": "0x3::account_coin_store::AutoAcceptCoins",
                    "value": {
                      "auto_accept_coins": {
                        "abilities": 4,
                        "type": "0x2::table::Table<address, bool>",
                        "value": {
                          "handle": "0x0db9d01a4f72e3708665fde27fc0da6cf353b0a9b0a2f3a2c2597f3e949e62d8"
                        }
                      }
                    }
                  }
                }
              }
            ],
                "next_cursor": {
                  "key": "0x65303030303030303030303030303030303030303030303030303030303030303030303030303030303030303030303030303030303030303030303030303030333a3a6163636f756e745f636f696e5f73746f72653a3a4175746f416363657074436f696e73",
                  "key_type": "0x1::ascii::String"
            },
            "has_next_page": true
          },
            "id": 101
          }

          mock.post('*', JSON.stringify(body))

          return mock
        })

        const provider = new RoochClient(DevChain, {
          fetcher: mockFetch,
        })
        expect(provider).toBeDefined()

        try {
          const assetsPath =
            '/table/0x030d80ff6a6b1a2467dffd11a7f0eba7d2b1a486289c3484112ca1245645dfe0'
          const cursor = null;
          const result = await provider.listStates(assetsPath, cursor, 1)

          expect(result.data).toBeDefined()
          expect(result.next_cursor).toBe(
              {
                "key": "0x65303030303030303030303030303030303030303030303030303030303030303030303030303030303030303030303030303030303030303030303030303030333a3a6163636f756e745f636f696e5f73746f72653a3a4175746f416363657074436f696e73",
                "key_type": "0x1::ascii::String"
              },
          )
          expect(result.has_next_page).toBeFalsy()
        } catch (err: any) {
          expect(err).to.be.an('error')
        }
      })
    })
  })
})
