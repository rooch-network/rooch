// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

import { Readable } from 'stream'

export class LogConsumer {
  public async waitUntilReady(stream: Readable): Promise<void> {
    stream.on('data', (line) => {
      console.log(`[Container Log] ${line.toString().trim()}`)
    })
  }
}
