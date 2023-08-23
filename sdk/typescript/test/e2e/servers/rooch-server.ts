import { spawn, ChildProcess } from 'child_process'

export class RoochServer {
  private child: ChildProcess | undefined

  private ready: boolean = false

  async start() {
    this.child = spawn('cargo', [
      'run',
      '--bin',
      'rooch',
      'server',
      'start',
      '--temp-db',
    ])

    if (this.child) {
      this.child.stdout?.on('data', (data) => {
        process.stdout.write(`${data}`)
      })

      this.child.stderr?.on('data', (data) => {
        process.stderr.write(`${data}`)
      })

      this.child.on('close', (code) => {
        if (code !== 0) {
          process.stderr.write(`子进程退出,退出码 ${code}`)
        }
      })

      this.child.on('error', (err) => {
        throw err
      })
    }

    await this.waitReady()
  }

  checkReady(cb: (ret: boolean) => void) {
    const readyRegex = /JSON-RPC HTTP Server start listening/

    this.child?.stdout?.on('data', (data) => {
      const text = data.toString()
      if (readyRegex.test(text)) {
        if (cb) {
          cb(true)
        }
      }
    })

    setTimeout(() => {
      if (cb) {
        cb(false)
      }
    }, 1000 * 300)
  }

  async waitReady(): Promise<void> {
    if (this.ready) {
      return
    }

    await new Promise<void>(
      (
        resolve: (value: void | PromiseLike<void>) => void,
        reject: (reason?: any) => void,
      ) => {
        this.checkReady((ready: boolean) => {
          if (ready) {
            this.ready = true
            resolve()
            return
          }

          reject(new Error('timeout'))
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
