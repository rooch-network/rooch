// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

import { Readable } from 'stream'

export function logConsumer(name: string) {
  return (stream: Readable): Promise<void> => {
    const topic = `Container ${name} log`

    return new Promise((resolve, reject) => {
      stream.on('data', (line) => {
        console.log(`[${topic}] ${line.toString().trim()}`)
      })

      stream.on('end', () => resolve())
      stream.on('error', (err) => reject(err))
    })
  }
}
