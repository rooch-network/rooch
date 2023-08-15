import { describe, it, expect, vi } from 'vitest'
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
            "error":
            {
                "code": -32000,
                "message": "VMError with status LINKER_ERROR at location UNDEFINED and message Cannot find ModuleId { address: 0c74a6f9397b0eacd8fec9e95e9dc671da9ec69d3088688226070183edf84516, name: Identifier(\"counter\") } in data cache"
            },
            "id": "0"
        };
        let init = {
            status: 200,
            statusText: "OK",
            headers: {
                'Content-Type': 'application/json'
            }
        };

        //let response = new Response(JSON.stringify(body), init);
        mockFetch.post("*", JSON.stringify(body))


        const wrapFetch = async (input: any, init: any) => {
            console.log("mockFetch: input:", input, "init:", init)
            const result = mockFetch(input, init)
            //console.log("mockFetch: result:", await result.text())
            return result
        }


        const provider = new JsonRpcProvider(LocalnetConnection, {
            fetcher: wrapFetch//mockFetch as (typeof fetch),
        })
        expect(provider).toBeDefined()


        let result = await provider.executeViewFunction("0xc74a6f9397b0eacd8fec9e95e9dc671da9ec69d3088688226070183edf84516::counter::value")
        expect(result).toHaveLength(1)


        /*
        const resp = await wrapFetch('http://127.0.0.1:50051', {
            method: 'POST',
            body: '{"jsonrpc":"2.0","id":"0","method":"rooch_executeViewFunction","params":[{"function_id":"0xc74a6f9397b0eacd8fec9e95e9dc671da9ec69d3088688226070183edf84516::counter::value","args":[],"ty_args":[]}]}',
        })

        const text = await resp.text()
        expect(text).toContain('VMError')
        */
    })
})
