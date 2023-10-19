// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

import { mimcHash } from '@darkforest_eth/hashing'
import * as snarkjs from 'snarkjs'
import { Location } from '../types'
import { loadJSON } from '../utils/http'

const worldWidth = 100
const worldHeight = 100

export async function makeJWTProof(
  publicURL: string,
  playerFogSeed: number,
  fogSeed: number,
  playerPosition: Location,
  px: number,
  py: number,
): Promise<any> {
  const jsCoordHash1 = mimcHash(playerFogSeed)(playerPosition.x, playerPosition.y).toString(10)
  const jsCoordHash2 = mimcHash(fogSeed)(px, py).toString(10)

  const { proof, publicSignals } = await snarkjs.groth16.fullProve(
    {
      x1: playerPosition.x,
      y1: playerPosition.y,
      x2: px,
      y2: py,
      seed1: playerFogSeed,
      seed2: fogSeed,
      width: worldWidth,
      height: worldHeight,
    },
    `${publicURL}/assets/circuits/jwt/circuit_js/circuit.wasm`,
    `${publicURL}/assets/circuits/jwt/circuit_0001.zkey`,
  )

  if (jsCoordHash1.toString() !== publicSignals[0]) {
    throw new Error('Coord hash not match: ' + jsCoordHash1 + ', ' + publicSignals[0])
  }

  if (jsCoordHash2.toString() !== publicSignals[1]) {
    throw new Error('Coord hash not match: ' + jsCoordHash2 + ', ' + publicSignals[0])
  }

  const zKey = await loadJSON(`${publicURL}/assets/circuits/jwt/verification_key.json`)

  const res = await snarkjs.groth16.verify(zKey, publicSignals, proof)
  if (res === false) {
    throw new Error('Invalid init proof')
  }

  const jwtInfo = {
    oldHash: publicSignals[0],
    coordHash: publicSignals[1],
    distance: publicSignals[2],
    oldSeed: publicSignals[3],
    seed: publicSignals[4],
    width: publicSignals[5],
    height: publicSignals[6],
    a: [proof.pi_a[0], proof.pi_a[1]],
    b: [
      [proof.pi_b[0][1], proof.pi_b[0][0]],
      [proof.pi_b[1][1], proof.pi_b[1][0]],
    ],
    c: [proof.pi_c[0], proof.pi_c[1]],
  }

  return jwtInfo
}
