import { describe, it, expect, vi } from 'vitest'
import fetchMock from 'fetch-mock'
import { LocalnetConnection } from './connection'
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
              move_value: '0',
            },
          ],
          id: '0',
        }

        mock.post('*', JSON.stringify(body))

        return mock
      })

      const provider = new JsonRpcProvider(LocalnetConnection, {
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

      const provider = new JsonRpcProvider(LocalnetConnection, {
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
  })
})
