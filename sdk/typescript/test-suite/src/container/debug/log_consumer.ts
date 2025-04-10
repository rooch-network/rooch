// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

import debug from 'debug'
import { Readable } from 'stream'

/**
 * Logs the output of a stream to the console.
 *
 * @param name - The name of the container.
 * @returns A function that takes a Readable stream and returns a Promise.
 */
export function logConsumer(name: string) {
  const log = debug(`test-suite:container:${name}`)

  return (stream: Readable): Promise<void> => {
    return new Promise((resolve, reject) => {
      stream.on('data', (line) => {
        log(`${line.toString().trim()}`)
      })

      stream.on('end', () => resolve())
      stream.on('error', (err) => reject(err))
    })
  }
}
