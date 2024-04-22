// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0
import { spawn, ChildProcess } from 'child_process'

export const DefaultPort = 50051

export class RoochServer {
  private child: ChildProcess | undefined

  private debug: boolean = false
  private ready: boolean = false
  private port: number = DefaultPort

  constructor(port: number = DefaultPort) {
    this.port = port
  }

  async start() {
    this.child = spawn('cargo', [
      'run',
      '--bin',
      'rooch',
      'server',
      'start',
      '-n',
      'local',
      '-d',
      'TMP',
      '--port',
      `${this.port}`,
    ])

    if (this.child) {
      this.child.stdout?.on('data', (data) => {
        if (this.debug) {
          process.stdout.write(`${data}`)
        }
      })

      this.child.stderr?.on('data', (data) => {
        if (this.debug) {
          process.stdout.write(`${data}`)
        }
      })

      this.child.on('close', (code) => {
        if (code !== 0) {
          process.stderr.write(`Child process exit, exit code ${code}`)
        }
      })

      this.child.on('error', (err) => {
        throw err
      })
    }

    await this.waitReady()
  }

  checkReady(cb: (ret: Error | undefined) => void) {
    const readyRegex = /JSON-RPC HTTP Server start listening/
    // Wait eth relayer ready, TODO: Direct inspection timestamp contract
    // const readyRegex = /EthereumRelayer process block/
    const errorRegex = /[Ee]rror:/

    const timer = setTimeout(() => {
      if (cb) {
        cb(new Error('timeout'))
      }
    }, 1000 * 300)

    this.child?.stdout?.on('data', (data) => {
      const text = data.toString()
      if (readyRegex.test(text)) {
        clearTimeout(timer)

        if (cb) {
          cb(undefined)
        }
      } else if (errorRegex.test(text)) {
        clearTimeout(timer)

        if (cb) {
          cb(new Error(text))
        }
      }
    })

    this.child?.stderr?.on('data', (text) => {
      if (errorRegex.test(text)) {
        clearTimeout(timer)

        if (cb) {
          cb(new Error(text))
        }
      }
    })

    this.child?.on('error', (err) => {
      clearTimeout(timer)

      if (cb) {
        cb(err)
      }
    })
  }

  async waitReady(): Promise<void> {
    if (this.ready) {
      return
    }

    await new Promise<void>(
      (resolve: (value: void | PromiseLike<void>) => void, reject: (reason?: any) => void) => {
        this.checkReady((err: Error | undefined) => {
          if (err) {
            reject(err)
            return
          }

          this.ready = true
          resolve()
        })
      },
    )
  }

  async stop() {
    if (this.child) {
      this.child.kill()
    }
  }
}
