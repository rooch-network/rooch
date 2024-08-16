// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

import { beforeAll, describe, it, expect } from 'vitest'

import { TestBox } from '../setup.js'

describe('Module Abi API', () => {
  let testBox: TestBox

  beforeAll(async () => {
    testBox = TestBox.setup()
  })

  it('Get module abi', async () => {
    const result = await testBox.getClient().getModuleAbi({
      moduleAddr: '0x3',
      moduleName: 'session_key',
    })

    expect(result).toBeDefined()
  })
})
