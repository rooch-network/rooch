import { describe, it, expect, vi} from 'vitest'
import { LocalnetConnection } from './connection'
import { JsonRpcProvider } from './json-rpc-provider'
import fetch from "isomorphic-fetch";
import fetchMock from 'fetch-mock';

describe('provider', () => {
    it('should create JsonRpcProvider ok ', async () => {
        const provider = new JsonRpcProvider()
        expect(provider).toBeDefined()
    })

    it('should execute view function', async () => {
        const injectedFetchMock = vi.fn().withImplementation(fetch, ()=>{
            const mock = fetchMock.sandbox()

            let body = { data: 'value' };
            let init = {
                status: 200,
                statusText: "OK",
                headers: {
                    'Content-Type': 'application/json'
                }
            };

            let response = new Response(JSON.stringify(body), init);
            mock.mock("*", response, {
                method: "GET"
            })
            
            return mock
        })

        const provider = new JsonRpcProvider(LocalnetConnection, {
            fetcher: injectedFetchMock,
        })
        expect(provider).toBeDefined()

        let result = await provider.executeViewFunction("0xc74a6f9397b0eacd8fec9e95e9dc671da9ec69d3088688226070183edf84516::counter::value")
        expect(result).toHaveLength(1)
    })
})
