// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0
import { spawn, ChildProcess } from 'child_process'

export class RoochCli {
  private debug: boolean = false
  private child: ChildProcess | undefined

  async execute(args: Array<string>): Promise<any> {
    return new Promise((resolve, reject) => {
      let output = ''
      let errorMsg = ''
      this.child = spawn('cargo', ['run', '--bin', 'rooch'].concat(args))

      if (this.child) {
        this.child.stdout?.on('data', (data) => {
          output += data

          if (this.debug) {
            process.stdout.write(`${data}`)
          }
        })

        this.child.stderr?.on('data', (data) => {
          errorMsg += data

          if (this.debug) {
            process.stderr.write(`${data}`)
          }
        })

        this.child.on('close', (code) => {
          if (code !== 0) {
            process.stderr.write(`Child process exit, exit code ${code}`)
            reject(new Error(`Child process exited with code ${code}, error message: ${errorMsg}`))
          } else {
            try {
              const jsonText = this.extractJSON(output)
              const json = JSON.parse(jsonText)
              resolve(json)
            } catch (err) {
              resolve(output)
            }
          }
        })

        this.child.on('error', (err) => {
          reject(err)
        })
      }
    })
  }

  extractJSON(output: string): string {
    const lines = output.trim().split('\n')
    let jsonStartIndex = -1

    // Find the line where JSON starts (first occurrence of '{')
    for (let i = 0; i < lines.length; i++) {
      if (lines[i].includes('{') || lines[i].includes('[')) {
        jsonStartIndex = i
        break
      }
    }

    // If '{' was found, join all lines from there to the end into a single string
    if (jsonStartIndex !== -1) {
      return lines.slice(jsonStartIndex).join('')
    }

    // If no '{' was found, return the original output
    return output
  }

  /**
   * Retrieves the default account address.
   *
   * This method lists all accounts and returns the address of the first active account found.
   * If no active account is present, it throws an error.
   *
   * @returns {Promise<string>} A promise that resolves with the address of the default account.
   * @throws {Error} When no active account address is found.
   */
  public async defaultAccountAddress(): Promise<string> {
    const accounts = await this.execute(['account', 'list', '--json'])
    for (const account of accounts) {
      if (account.active) {
        return account.local_account.address
      }
    }

    throw new Error('No active account address')
  }
}
