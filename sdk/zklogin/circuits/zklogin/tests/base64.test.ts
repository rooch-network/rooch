// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

import { wasm as wasm_tester } from 'circom_tester'
import path from 'path'
import { padString } from '../src'

describe('Base64', () => {
  jest.setTimeout(10 * 60 * 1000) // 10 minutes

  let circuit: any

  describe('Lookup', () => {
    beforeAll(async () => {
      circuit = await wasm_tester(path.join(__dirname, './base64-test.circom'), {
        // @dev During development recompile can be set to false if you are only making changes in the tests.
        // This will save time by not recompiling the circuit every time.
        // Compile: circom "./tests/email-verifier-test.circom" --r1cs --wasm --sym --c --wat --output "./tests/compiled-test-circuit"
        recompile: true,
        output: path.join(__dirname, './compiled-test-circuit'),
        include: path.join(__dirname, '../../../node_modules'),
      })
    })

    it('should decode valid base64 chars', async function () {
      const inputs = [
        [65, 0], // A
        [90, 25], // Z
        [97, 26], // a
        [122, 51], // z
        [48, 52], // 0
        [57, 61], // 9
        [43, 62], // +
        [47, 63], // /
        [61, 0], // =
      ]

      for (const [input, output] of inputs) {
        const witness = await circuit.calculateWitness({
          in: input,
        })
        await circuit.checkConstraints(witness)
        await circuit.assertOut(witness, { out: output })
      }
    })

    it('should fail with invalid chars', async function () {
      const inputs = [34, 64, 91, 44]

      expect.assertions(inputs.length)
      for (const input of inputs) {
        try {
          const witness = await circuit.calculateWitness({
            in: input,
          })
          await circuit.checkConstraints(witness)
        } catch (error) {
          expect((error as Error).message).toMatch('Assert Failed')
        }
      }
    })
  })

  describe('Base64Decode', () => {
    beforeAll(async () => {
      circuit = await wasm_tester(path.join(__dirname, './base64-decode-test.circom'), {
        // @dev During development recompile can be set to false if you are only making changes in the tests.
        // This will save time by not recompiling the circuit every time.
        // Compile: circom "./tests/email-verifier-test.circom" --r1cs --wasm --sym --c --wat --output "./tests/compiled-test-circuit"
        recompile: true,
        output: path.join(__dirname, './compiled-test-circuit'),
        include: path.join(__dirname, '../../../node_modules'),
      })
    })

    it('should decode base64 be ok', async function () {
      const inputs = [
        [
          padString('eyJhbGciOiJSUzI1NiIsInR5cCI6IkpXVCJ9', 128),
          padString('{"alg":"RS256","typ":"JWT"}', 128),
        ],
        [
          padString(
            'eyJzdWIiOiIxMjM0NTY3ODkwIiwibmFtZSI6IkpvaG4gRG9lIiwiYWRtaW4iOnRydWUsImlhdCI6MTUxNjIzOTAyMn0',
            128,
          ),
          padString('{"sub":"1234567890","name":"John Doe","admin":true,"iat":1516239022}', 128),
        ],
      ]

      for (const [input, output] of inputs) {
        const witness = await circuit.calculateWitness({
          in: input,
        })
        await circuit.checkConstraints(witness)
        await circuit.assertOut(witness, { out: output })
      }
    })
  })
})
