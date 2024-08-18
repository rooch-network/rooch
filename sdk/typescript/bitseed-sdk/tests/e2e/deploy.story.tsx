// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0
import React, { useEffect, useState } from 'react'
import {
  BitSeed,
  InscriptionID,
  parseInscriptionID,
  inscriptionIDToString,
  DeployOptions,
} from '../../src'

import { createTestBitSeed } from './commons/test_bitseed_web'

interface DeployStoryProps {
  roochServerAddress: string
}

export default function DeployStory({ roochServerAddress }: DeployStoryProps) {
  const [bitseed, setBitseed] = useState<BitSeed | undefined>(undefined)
  const [tick, setTick] = useState<string>('')
  const [max, setMax] = useState<number>(0)
  const [generatorValue, setGeneratorValue] = useState<string>('')
  const [deployArg, setDeployArg] = useState<string>('')
  const [deployResult, setDeployResult] = useState<InscriptionID | undefined>(undefined)
  const [error, setError] = useState<string | undefined>(undefined)

  useEffect(() => {
    setBitseed(createTestBitSeed(roochServerAddress))
  }, [roochServerAddress])

  const handleDeploy = async () => {
    if (!bitseed) return

    try {
      let generator = parseInscriptionID(generatorValue)
      const deployArgs = [deployArg]

      const deployOptions: DeployOptions = {
        fee_rate: 1,
        repeat: 1,
        deploy_args: deployArgs,
      }

      const inscriptionId = await bitseed.deploy(tick, max, generator, deployOptions)
      setDeployResult(inscriptionId)
      setError(undefined)
    } catch (e) {
      setError(e.message)
      setDeployResult(undefined)
    }
  }

  return (
    <div>
      <div>Deploy Tick (Rooch Server: {roochServerAddress})</div>
      <div>
        Tick:{' '}
        <input
          type="text"
          placeholder="Tick"
          value={tick}
          onChange={(e) => setTick(e.target.value)}
        />
        <br />
        Max:{' '}
        <input
          type="number"
          placeholder="Max"
          value={max}
          onChange={(e) => setMax(Number(e.target.value))}
        />
        <br />
        GeneratorInscriptionID:{' '}
        <input
          type="text"
          placeholder="GeneratorInscriptionID"
          value={generatorValue}
          onChange={(e) => setGeneratorValue(e.target.value)}
        />
        <br />
        DeployArg:{' '}
        <input
          type="text"
          placeholder="DeployArg"
          value={deployArg}
          onChange={(e) => setDeployArg(e.target.value)}
        />
        <br />
        <button onClick={handleDeploy}>Deploy</button>
      </div>
      {deployResult && <div>Deploy Result: {inscriptionIDToString(deployResult)}</div>}
      {error && <div>Error: {error}</div>}
    </div>
  )
}
