// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

import { spawn } from 'child_process'

export const sh = (cmd, args) => {
  return new Promise((resolve, reject) => {
    let child = spawn(cmd, args)

    child.stdout.on('data', (data) => {
      process.stdout.write(`${data}`)
    })

    child.stderr.on('data', (data) => {
      process.stderr.write(`${data}`)
    })

    child.on('close', (code) => {
      if (code !== 0) {
        process.stderr.write(`子进程退出，退出码 ${code}`)
      }

      resolve(code)
    })

    child.on('error', (err) => {
      reject(err)
    })
  })
}
