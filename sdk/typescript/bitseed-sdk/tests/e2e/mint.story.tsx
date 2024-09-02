// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0
import React, { useEffect, useState } from 'react'

import {
  BitSeed,
  InscriptionID,
  parseInscriptionID,
  inscriptionIDToString,
  InscribeOptions,
  hexStringToTxid,
} from '../../src'
import { createTestBitSeed } from './commons/test_bitseed_web'

interface MintStoryProps {
  roochServerAddress: string
}

export default function MintStory({ roochServerAddress }: MintStoryProps) {
  const [bitseed, setBitseed] = useState<BitSeed | undefined>(undefined)

  const [tickDeployInscriptionID, setTickDeployInscriptionID] = useState<string>('')
  const [userInput, setUserInput] = useState<string>('')

  const [mintResult, setMintResult] = useState<InscriptionID | undefined>(undefined)
  const [error, setError] = useState<string | undefined>(undefined)

  useEffect(() => {
    setBitseed(createTestBitSeed(roochServerAddress))
  }, [roochServerAddress])

  const handleMint = async () => {
    if (!bitseed) return

    try {
      let tick = parseInscriptionID(tickDeployInscriptionID)
      const mintOptions: InscribeOptions = {
        fee_rate: 1,
        satpoint: {
          outpoint: {
            txid: hexStringToTxid(tick.txid),
            vout: 0,
          },
          offset: 0,
        },
      }

      console.log(`handleMint tick:${tick}, userInput:${userInput}, mintOptions:${mintOptions}`)
      const inscriptionId = await bitseed.mint(tick, userInput, mintOptions)

      setMintResult(inscriptionId)
      setError(undefined)
    } catch (e) {
      setError(e.message)
      setMintResult(undefined)
    }
  }

  return (
    <div>
      <div>Mint Tick</div>
      <div>
        TickDeployID:{' '}
        <input
          type="text"
          placeholder="TickDeployID"
          value={tickDeployInscriptionID}
          onChange={(e) => setTickDeployInscriptionID(e.target.value)}
        />{' '}
        <br />
        UserInput:{' '}
        <input
          type="text"
          placeholder="UserInput"
          value={userInput}
          onChange={(e) => setUserInput(e.target.value)}
        />{' '}
        <br />
        <button onClick={handleMint}>Mint</button>
      </div>
      {mintResult && <div>Mint Result: {inscriptionIDToString(mintResult)}</div>}
      {error && <div>Error: {error}</div>}
    </div>
  )
}
