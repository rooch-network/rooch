import { describe, it, expect, beforeAll, afterAll } from 'vitest'
import { JsonRpcProvider  } from '../../src'
import { RoochServer } from './servers/rooch-server';

describe('viewFunction', () => {
  let server: RoochServer;

  beforeAll(async () => {
    server = new RoochServer();
    await server.start();
  });

  afterAll(async () => {
    await server.stop();
  });

  it('view function should be ok', async () => {
    const provider = new JsonRpcProvider()
    const result = await provider.executeViewFunction('0x1::account::sequence_number_for_sender', [], [])
    expect(result).toBeDefined()
  })
})
