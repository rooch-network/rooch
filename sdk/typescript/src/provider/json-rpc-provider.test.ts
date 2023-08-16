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
        const mockFetch = vi.fn().mockImplementation(()=>{
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

            mockFetch.post("*", JSON.stringify(body))

            return mockFetch
        })
        
        const provider = new JsonRpcProvider(LocalnetConnection, {
            fetcher: mockFetch,
        })
        expect(provider).toBeDefined()

        try {
            let result = await provider.executeViewFunction("0xc74a6f9397b0eacd8fec9e95e9dc671da9ec69d3088688226070183edf84516::counter::value")
            expect(result).toHaveLength(1)
        } catch(err: any) {
            expect(err).to.be.an('error'); 
        }
    })
    
    it('should execute view function', async () => {
        const mockFetch = vi.fn().mockImplementation(()=>{
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

            mockFetch.post("*", JSON.stringify(body))

            return mockFetch
        })
        
        const provider = new JsonRpcProvider(LocalnetConnection, {
            fetcher: mockFetch,
        })
        expect(provider).toBeDefined()

        try {
            let result = await provider.executeViewFunction("0xc74a6f9397b0eacd8fec9e95e9dc671da9ec69d3088688226070183edf84516::counter::value")
            expect(result).toHaveLength(1)
        } catch(err: any) {
            expect(err).to.be.an('error'); 
        }
    })
})
