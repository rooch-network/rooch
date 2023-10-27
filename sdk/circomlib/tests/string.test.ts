// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

import { wasm as wasm_tester } from 'circom_tester'
import path from 'path'
import { Felt, padString } from '../src'

describe('String Test', () => {
  jest.setTimeout(10 * 60 * 1000) // 10 minutes

  let circuit: any

  describe('Len', () => {
    beforeAll(async () => {
      circuit = await wasm_tester(path.join(__dirname, './string-len-test.circom'), {
        // @dev During development recompile can be set to false if you are only making changes in the tests.
        // This will save time by not recompiling the circuit every time.
        // Compile: circom "./tests/email-verifier-test.circom" --r1cs --wasm --sym --c --wat --output "./tests/compiled-test-circuit"
        recompile: true,
        output: path.join(__dirname, './compiled-test-circuit'),
        include: path.join(__dirname, '../node_modules'),
      })
    })

    it('should len be ok', async function () {
      const inputs = [
        [padString('A', 256), 1],
        [padString('AB', 256), 2],
      ]

      for (const [text, output] of inputs) {
        const witness = await circuit.calculateWitness({
          text: text,
        })
        await circuit.checkConstraints(witness)
        await circuit.assertOut(witness, { length: output })
      }
    })
  })

  describe('CharAt', () => {
    beforeAll(async () => {
      circuit = await wasm_tester(path.join(__dirname, './string-charat-test.circom'), {
        // @dev During development recompile can be set to false if you are only making changes in the tests.
        // This will save time by not recompiling the circuit every time.
        // Compile: circom "./tests/email-verifier-test.circom" --r1cs --wasm --sym --c --wat --output "./tests/compiled-test-circuit"
        recompile: true,
        output: path.join(__dirname, './compiled-test-circuit'),
        include: path.join(__dirname, '../node_modules'),
      })
    })

    it('should chatAt be ok', async function () {
      const inputs = [
        [padString('A', 256), 0, 65],
        [padString('AB', 256), 1, 66],
        [padString('AB', 256), 270, 0],
      ]

      for (const [text, index, output] of inputs) {
        const witness = await circuit.calculateWitness({
          text: text,
          index,
        })
        await circuit.checkConstraints(witness)
        await circuit.assertOut(witness, { ch: output })
      }
    })
  })

  describe('IndexOf', () => {
    beforeAll(async () => {
      circuit = await wasm_tester(path.join(__dirname, './string-indexof-test.circom'), {
        // @dev During development recompile can be set to false if you are only making changes in the tests.
        // This will save time by not recompiling the circuit every time.
        // Compile: circom "./tests/email-verifier-test.circom" --r1cs --wasm --sym --c --wat --output "./tests/compiled-test-circuit"
        recompile: true,
        output: path.join(__dirname, './compiled-test-circuit'),
        include: path.join(__dirname, '../node_modules'),
      })
    })

    it('should IndexOf with startIndex zero be ok', async function () {
      const inputs = [
        [padString('A', 256), 'A'.charCodeAt(0), 0], // A
        [padString('AB', 256), 'B'.charCodeAt(0), 1], // B
        [padString('AB', 256), 'C'.charCodeAt(0), Felt.NEGATIVE_ONE], // B
      ]

      for (const [text, targetChar, output] of inputs) {
        const witness = await circuit.calculateWitness({
          text,
          startIndex: 0,
          targetChar,
        })
        await circuit.checkConstraints(witness)
        await circuit.assertOut(witness, { index: output })
      }
    })

    it('should IndexOf with startIndex gt zero be ok', async function () {
      const inputs = [
        [padString('ABCABC', 256), 3, 'C'.charCodeAt(0), 5],
        [padString('ABC.ABC.ABC', 256), 0, '.'.charCodeAt(0), 3],
      ]

      for (const [text, startIndex, targetChar, output] of inputs) {
        const witness = await circuit.calculateWitness({
          text,
          startIndex,
          targetChar,
        })
        await circuit.checkConstraints(witness)
        await circuit.assertOut(witness, { index: output })
      }
    })
  })

  describe('SubString', () => {
    beforeAll(async () => {
      circuit = await wasm_tester(path.join(__dirname, './string-substring-test.circom'), {
        // @dev During development recompile can be set to false if you are only making changes in the tests.
        // This will save time by not recompiling the circuit every time.
        // Compile: circom "./tests/email-verifier-test.circom" --r1cs --wasm --sym --c --wat --output "./tests/compiled-test-circuit"
        recompile: true,
        output: path.join(__dirname, './compiled-test-circuit'),
        include: path.join(__dirname, '../node_modules'),
      })
    })

    it('should SubString be ok', async function () {
      const inputs = [
        [padString('ABCDEFG', 256), 0, 3, padString('ABC', 64)],
        [padString('ABCDEFG', 256), 1, 3, padString('BCD', 64)],
        [padString('ABC.ABC.ABC', 256), 0, 3, padString('ABC', 64)],
      ]

      for (const [text, startIndex, count, output] of inputs) {
        const witness = await circuit.calculateWitness({
          text,
          startIndex,
          count,
        })
        await circuit.checkConstraints(witness)
        await circuit.assertOut(witness, { substring: output })
      }
    })
  })

  describe('Split', () => {
    beforeAll(async () => {
      circuit = await wasm_tester(path.join(__dirname, './string-split-test.circom'), {
        // @dev During development recompile can be set to false if you are only making changes in the tests.
        // This will save time by not recompiling the circuit every time.
        // Compile: circom "./tests/email-verifier-test.circom" --r1cs --wasm --sym --c --wat --output "./tests/compiled-test-circuit"
        recompile: true,
        output: path.join(__dirname, './compiled-test-circuit'),
        include: path.join(__dirname, '../node_modules'),
      })
    })

    it('should chatAt be ok', async function () {
      const inputs = [
        [
          padString('ABC.ABC.ABC', 256),
          [padString('ABC', 256), padString('ABC', 256), padString('ABC', 256)],
        ],
        [
          padString('ABC.ABCDF.ABCEFG', 256),
          [padString('ABC', 256), padString('ABCDF', 256), padString('ABCEFG', 256)],
        ],
      ]

      for (const [text, output] of inputs) {
        const witness = await circuit.calculateWitness({
          text,
        })
        await circuit.checkConstraints(witness)
        await circuit.assertOut(witness, { out: output })
      }
    })
  })

  describe('Concat', () => {
    beforeAll(async () => {
      circuit = await wasm_tester(path.join(__dirname, './string-concat-test.circom'), {
        // @dev During development recompile can be set to false if you are only making changes in the tests.
        // This will save time by not recompiling the circuit every time.
        // Compile: circom "./tests/email-verifier-test.circom" --r1cs --wasm --sym --c --wat --output "./tests/compiled-test-circuit"
        recompile: true,
        output: path.join(__dirname, './compiled-test-circuit'),
        include: path.join(__dirname, '../node_modules'),
      })
    })

    it('should concat be ok', async function () {
      const inputs = [[padString('ABC', 4), padString('DEFGHI', 8), padString('ABCDEFGHI', 12)]]

      for (const [text1, text2, output] of inputs) {
        const witness = await circuit.calculateWitness({
          text1,
          text2,
        })
        await circuit.checkConstraints(witness)
        await circuit.assertOut(witness, { out: output })
      }
    })
  })

  describe('Concat3', () => {
    beforeAll(async () => {
      circuit = await wasm_tester(path.join(__dirname, './string-concat3-test.circom'), {
        // @dev During development recompile can be set to false if you are only making changes in the tests.
        // This will save time by not recompiling the circuit every time.
        // Compile: circom "./tests/email-verifier-test.circom" --r1cs --wasm --sym --c --wat --output "./tests/compiled-test-circuit"
        recompile: true,
        output: path.join(__dirname, './compiled-test-circuit'),
        include: path.join(__dirname, '../node_modules'),
      })
    })

    it('should concat be ok', async function () {
      const inputs = [
        [
          padString('ABC', 4),
          padString('.', 2),
          padString('DEFGHI', 8),
          padString('ABC.DEFGHI', 14),
        ],
      ]

      for (const [text1, text2, text3, output] of inputs) {
        const witness = await circuit.calculateWitness({
          text1,
          text2,
          text3,
        })
        await circuit.checkConstraints(witness)
        await circuit.assertOut(witness, { out: output })
      }
    })
  })
})
