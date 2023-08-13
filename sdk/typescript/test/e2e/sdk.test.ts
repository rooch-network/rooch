import { describe, it, expect } from 'vitest'
import { JsonRpcProvider } from '../../src'

describe('provier', () => {
  it('should execute view function', async () => {
    // Temporary test
    // TODO: instead e2e
    const jrp = new JsonRpcProvider()
    expect(jrp).toBeDefined()
    //let result = await jrp.executeViewFunction("0xc74a6f9397b0eacd8fec9e95e9dc671da9ec69d3088688226070183edf84516::counter::value")
    //expect(result).toHaveLength(1)
  })

  it('should sign and execute function', () => {
    const jrp = new JsonRpcProvider()
  })
})
