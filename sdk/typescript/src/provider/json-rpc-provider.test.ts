import { describe, it, expect, vi} from 'vitest'
import { LocalnetConnection } from './connection'
import { JsonRpcProvider } from './json-rpc-provider'
import fetch from "isomorphic-fetch"
import fetchMock from 'fetch-mock';

describe('provider', () => {
    it('should create JsonRpcProvider ok ', async () => {
        const provider = new JsonRpcProvider()
        expect(provider).toBeDefined()
    })

    it('should execute view function', async () => {
        
        const mockFetch = fetchMock.sandbox()

        let body = {
            "jsonrpc": "2.0",
            "result": 19,
            "id": 1
        };
        let init = {
            status: 200,
            statusText: "OK",
            headers: {
                'Content-Type': 'application/json'
            }
        };

        let response = new Response(JSON.stringify(body), init);
        mockFetch.post("*", response, {
            method: "POST"
        })
        

        const wrapFetch = async (input:any, init:any)=>{
            console.log("mockFetch: input:", input, "init:", init)
            const result = await fetch(input, init)
            console.log("mockFetch: result:", result)
            return result
        }

        const provider = new JsonRpcProvider(LocalnetConnection, {
            fetcher: wrapFetch//mockFetch as (typeof fetch),
        })
        expect(provider).toBeDefined()

        let result = await provider.executeViewFunction("0xc74a6f9397b0eacd8fec9e95e9dc671da9ec69d3088688226070183edf84516::counter::value")
        expect(result).toHaveLength(1)
    })
})
