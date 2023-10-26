// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

import { Felt } from './felt'

describe('Felt', () => {
  jest.setTimeout(10 * 60 * 1000) // 10 minutes

  it('should decode valid base64 chars', async function () {
    const felt = Felt.fromString('Hello World')
    expect(felt.toText()).toBe('Hello World')
  })
})
