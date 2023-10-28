// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

import { wasm as wasm_tester } from 'circom_tester'
import path from 'path'
import { padString, sha256Pad, shaHash, Uint8ArrayToCharArray, uint8ToBits } from '../src'

describe('SHA256', () => {
  jest.setTimeout(10 * 60 * 1000) // 10 minutes

  let circuit: any

  describe('Sha256Bytes', () => {
    beforeAll(async () => {
      circuit = await wasm_tester(path.join(__dirname, './sha256-bytes-test.circom'), {
        // @dev During development recompile can be set to false if you are only making changes in the tests.
        // This will save time by not recompiling the circuit every time.
        // Compile: circom "./tests/email-verifier-test.circom" --r1cs --wasm --sym --c --wat --output "./tests/compiled-test-circuit"
        recompile: true,
        output: path.join(__dirname, './compiled-test-circuit'),
        include: path.join(__dirname, '../node_modules'),
      })
    })

    it('should hash correctly', async function () {
      let encoder = new TextEncoder()
      const inputs = [
        '0',
        'hello world',
        'eyJhbGciOiJSUzI1NiIsInR5cCI6IkpXVCJ9.eyJzdWIiOiIxMjM0NTY3ODkwIiwibmFtZSI6IkpvaG4gRG9lIiwiYWRtaW4iOnRydWUsImlhdCI6MTUxNjIzOTAyMn0',
        '',
      ]
      for (const input of inputs) {
        const [paddedMsg, messageLen] = sha256Pad(encoder.encode(input), 640)

        const witness = await circuit.calculateWitness({
          in_len_padded_bytes: messageLen,
          in_padded: Uint8ArrayToCharArray(paddedMsg),
        })

        await circuit.checkConstraints(witness)
        await circuit.assertOut(witness, {
          out: [...uint8ToBits(shaHash(encoder.encode(input)))],
        })
      }
    })
  })

  describe('Sha256Pad', () => {
    beforeAll(async () => {
      circuit = await wasm_tester(path.join(__dirname, './sha256-pad-test.circom'), {
        // @dev During development recompile can be set to false if you are only making changes in the tests.
        // This will save time by not recompiling the circuit every time.
        // Compile: circom "./tests/email-verifier-test.circom" --r1cs --wasm --sym --c --wat --output "./tests/compiled-test-circuit"
        recompile: true,
        output: path.join(__dirname, './compiled-test-circuit'),
        include: path.join(__dirname, '../node_modules'),
      })
    })

    it('should sha256Pad correctly', async function () {
      let encoder = new TextEncoder()
      const inputs = [
        'eyJhbGciOiJSUzI1NiIsInR5cCI6IkpXVCJ9.eyJzdWIiOiIxMjM0NTY3ODkwIiwibmFtZSI6IkpvaG4gRG9lIiwiYWRtaW4iOnRydWUsImlhdCI6MTUxNjIzOTAyMn0',
      ]

      for (const input of inputs) {
        const padText = padString(input, 640)
        const [paddedMsg, messageLen] = sha256Pad(encoder.encode(input), 640)

        const witness = await circuit.calculateWitness({
          text: padText,
        })

        await circuit.checkConstraints(witness)
        await circuit.assertOut(witness, {
          padded_len: messageLen,
          padded_text: Array.from(paddedMsg),
        })
      }
    })
  })

  describe('Sha256String', () => {
    beforeAll(async () => {
      circuit = await wasm_tester(path.join(__dirname, './sha256-string-test.circom'), {
        // @dev During development recompile can be set to false if you are only making changes in the tests.
        // This will save time by not recompiling the circuit every time.
        // Compile: circom "./tests/email-verifier-test.circom" --r1cs --wasm --sym --c --wat --output "./tests/compiled-test-circuit"
        recompile: true,
        output: path.join(__dirname, './compiled-test-circuit'),
        include: path.join(__dirname, '../node_modules'),
      })
    })

    it('should hash correctly', async function () {
      let encoder = new TextEncoder()
      const inputs = [
        'eyJhbGciOiJSUzI1NiIsInR5cCI6IkpXVCJ9.eyJzdWIiOiIxMjM0NTY3ODkwIiwibmFtZSI6IkpvaG4gRG9lIiwiYWRtaW4iOnRydWUsImlhdCI6MTUxNjIzOTAyMn0',
      ]
      for (const [input] of inputs) {
        const padText = padString(input, 640)
        const witness = await circuit.calculateWitness({
          text: padText,
        })

        await circuit.checkConstraints(witness)
        await circuit.assertOut(witness, {
          sha: [...uint8ToBits(shaHash(encoder.encode(input)))],
        })
      }
    })
  })
})
