// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

export async function getDecoratedOutputArray(
  wasm_tester: any,
  witness: any,
  prefix: string,
  n: number,
): Promise<any> {
  const lines = []

  if (!wasm_tester.symbols) await wasm_tester.loadSymbols()

  for (let i = 0; i < n; i++) {
    let v: any

    let key = `${prefix}[${i}]`
    if (witness[wasm_tester.symbols[key].varIdx]) {
      v = witness[wasm_tester.symbols[key].varIdx]
    } else {
      v = 'undefined'
    }

    lines.push(v)
  }

  return lines
}

export async function getDecoratedOutputValue(
  wasm_tester: any,
  witness: any,
  prefix: string,
): Promise<any> {
  let v: any

  if (!wasm_tester.symbols) await wasm_tester.loadSymbols()

  let key = `${prefix}`
  if (witness[wasm_tester.symbols[key].varIdx]) {
    v = witness[wasm_tester.symbols[key].varIdx]
  } else {
    v = 'undefined'
  }

  return v
}
