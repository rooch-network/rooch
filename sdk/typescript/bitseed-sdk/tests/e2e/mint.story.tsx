import React, { useEffect, useState } from 'react'

import {
  BitSeed,
  InscriptionID,
  parseInscriptionID,
  inscriptionIDToString,
  InscribeOptions,
} from '../../src'
import { createTestBitSeed } from './commons/test_bitseed'

export default function MintStory() {
  const [bitseed, setBitseed] = useState<BitSeed | undefined>(undefined)

  const [tickDeployInscriptionID, setTickDeployInscriptionID] = useState<string>('')
  const [userInput, setUserInput] = useState<string>('')

  const [mintResult, setMintResult] = useState<InscriptionID | undefined>(undefined)
  const [error, setError] = useState<string | undefined>(undefined)

  useEffect(() => {
    setBitseed(createTestBitSeed())
  }, [])

  const handleMint = async () => {
    if (!bitseed) return

    console.log('handle mint tick')

    try {
      let tick = parseInscriptionID(tickDeployInscriptionID)
      const mintOptions: InscribeOptions = {
        fee_rate: 1,
        satpoint: {
          outpoint: {
            txid: '42d186a5d9bc064e5704024afb2dfccd424da1b9756ae31a4fbfee22f4fc7ec5',
            vout: 0
          },
          offset: 0
        }
      }

      const inscriptionId = await bitseed.mint(tick, userInput, mintOptions)
      console.log('mint ok, inscriptionId:', inscriptionId)

      setMintResult(inscriptionId)
      setError(undefined)
    } catch (e) {
      console.log('mint error:', e)
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
