// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0
import { spawn, ChildProcess } from 'child_process'

export class RoochServer {
  private child: ChildProcess | undefined

  private ready: boolean = false

  async start() {
    this.child = spawn('cargo', ['run', '--bin', 'rooch', 'server', 'start'])

    if (this.child) {
      this.child.stdout?.on('data', (data) => {
        process.stdout.write(`${data}`)
      })

      this.child.stderr?.on('data', (data) => {
        process.stderr.write(`${data}`)
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
    const errorRegex = /[Ee]rror:/

    const timer = setTimeout(() => {
      if (cb) {
        cb(new Error('timeout'))
      }
    }, 1000 * 60)

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
