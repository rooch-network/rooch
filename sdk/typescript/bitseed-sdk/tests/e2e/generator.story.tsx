// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

import React, { useEffect, useState } from 'react'

import { BitSeed, InscriptionID, inscriptionIDToString, DeployOptions } from '../../src'
import { createTestBitSeed } from './commons/test_bitseed_web'

interface DeployGeneratorStoryProps {
  roochServerAddress: string
}

export default function DeployGeneratorStory({ roochServerAddress }: DeployGeneratorStoryProps) {
  const [bitseed, setBitseed] = useState<BitSeed | undefined>(undefined)
  const [file, setFile] = useState<File | null>(null)
  const [deployResult, setDeployResult] = useState<InscriptionID | undefined>(undefined)
  const [error, setError] = useState<string | undefined>(undefined)

  useEffect(() => {
    setBitseed(createTestBitSeed(roochServerAddress))
  }, [roochServerAddress])

  const handleDeploy = async () => {
    if (!bitseed) return

    if (!file) return

    try {
      let wasmBytes = await readFileAsBytes(file)

      const deployOptions: DeployOptions = {
        fee_rate: 1,
      }

      const inscriptionId = await bitseed.generator('simple', wasmBytes, deployOptions)
      setDeployResult(inscriptionId)
      setError(undefined)
    } catch (e) {
      setError(e.message)
      setDeployResult(undefined)
    }
  }

  const handleFileChange = (event: React.ChangeEvent<HTMLInputElement>) => {
    const files = event.target.files
    if (files && files.length > 0) {
      setFile(files[0])
    } else {
      setFile(null)
    }
  }

  // 读取文件内容并转换为 Uint8Array 的函数
  const readFileAsBytes = (file: File): Promise<Uint8Array> => {
    return new Promise((resolve, reject) => {
      const reader = new FileReader()
      reader.onload = (event) => {
        const result = event.target?.result
        if (result) {
          resolve(new Uint8Array(result as ArrayBuffer))
        } else {
          reject(new Error('Failed to read file'))
        }
      }
      reader.onerror = (event) => {
        reject(new Error(`FileReader error: ${event.target?.error?.message}`))
      }
      reader.readAsArrayBuffer(file)
    })
  }

  return (
    <div>
      <div>Deploy generator</div>
      <div>
        <input type="file" placeholder="wasmFile" onChange={handleFileChange} />
        <button onClick={handleDeploy}>Deploy</button>
      </div>
      {deployResult && <div>Deploy Result: {inscriptionIDToString(deployResult)}</div>}
      {error && <div>Error: {error}</div>}
    </div>
  )
}
