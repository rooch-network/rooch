// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

import { describe, it, expect, vi } from 'vitest'
import fetchMock from 'fetch-mock'
import { DevChain } from '../constants/chain'
import { JsonRpcProvider } from './json-rpc-provider'

describe('provider', () => {
  it('should create JsonRpcProvider ok ', async () => {
    const provider = new JsonRpcProvider()
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

      const provider = new JsonRpcProvider(DevChain, {
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

      const provider = new JsonRpcProvider(DevChain, {
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

        const provider = new JsonRpcProvider(DevChain, {
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
            jsonrpc: '2.0',
            result: {
              data: [
                {
                  state: {
                    value:
                      '0x0e526f6f63682047617320436f696e03524743090000000000000000000000000000000000000000000000000000000000000000',
                    value_type: '0x3::coin::CoinInfo<0x3::gas_coin::GasCoin>',
                    decoded_value: {
                      abilities: 8,
                      type: '0x3::coin::CoinInfo<0x3::gas_coin::GasCoin>',
                      value: {
                        decimals: 9,
                        name: 'Rooch Gas Coin',
                        supply: '0',
                        symbol: 'RGC',
                      },
                    },
                  },
                },
              ],
              next_cursor:
                '0xa501303030303030303030303030303030303030303030303030303030303030303030303030303030303030303030303030303030303030303030303030303030333a3a636f696e3a3a436f696e496e666f3c303030303030303030303030303030303030303030303030303030303030303030303030303030303030303030303030303030303030303030303030303030333a3a6761735f636f696e3a3a476173436f696e3e',
              has_next_page: false,
            },
            id: '0',
          }

          mock.post('*', JSON.stringify(body))

          return mock
        })

        const provider = new JsonRpcProvider(DevChain, {
          fetcher: mockFetch,
        })
        expect(provider).toBeDefined()

        try {
          const assetsPath =
            '/table/0x82af1915608fa5f3e5286e4372e289b5b3ef03d0126cdae9ca7f561a145359c8'
          const cursor = new Uint8Array([0])
          const result = await provider.listStates(assetsPath, cursor, 10)

          expect(result.data).toBeDefined()
          expect(result.next_cursor).toBe(
            '0xa501303030303030303030303030303030303030303030303030303030303030303030303030303030303030303030303030303030303030303030303030303030333a3a636f696e3a3a436f696e496e666f3c303030303030303030303030303030303030303030303030303030303030303030303030303030303030303030303030303030303030303030303030303030333a3a6761735f636f696e3a3a476173436f696e3e',
          )
          expect(result.has_next_page).toBeFalsy()
        } catch (err: any) {
          expect(err).to.be.an('error')
        }
      })
    })
  })
})
