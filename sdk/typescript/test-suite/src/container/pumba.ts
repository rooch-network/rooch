// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

import { GenericContainer } from 'testcontainers'

export class PumbaContainer {
  private static async runPumbaCommand(command: string[]): Promise<void> {
    console.log(`Executing Pumba command: ${command.join(' ')}`)

    const container = await new GenericContainer('gaiaadm/pumba:latest')
      .withBindMounts([{ source: '/var/run/docker.sock', target: '/var/run/docker.sock' }])
      .withCommand(command)
      .withStartupTimeout(120_000)
      .start()

    console.log('Pumba command started successfully')

    // Wait for the container to stop by itself
    const stream = await container.logs()
    return new Promise<void>((resolve, reject) => {
      stream.on('data', (line) => {
        console.log(`[Pumba] ${line.toString().trim()}`)
      })

      stream.on('end', () => {
        console.log('Pumba command completed')
        resolve()
      })

      stream.on('error', (err) => {
        console.error('Pumba command error:', err)
        reject(err)
      })
    })
  }

  /**
   * Simulates network delay for a target container.
   * @param targetContainerName Name of the container to target.
   * @param delayMs Delay in milliseconds.
   * @param durationSec Duration of the fault in seconds.
   */
  public static async simulateDelay(
    targetContainerName: string,
    delayMs: number,
    durationSec: number,
  ): Promise<void> {
    await this.runPumbaCommand([
      'netem',
      '--duration',
      `${durationSec}s`,
      'delay',
      '--time',
      `${delayMs}`,
      targetContainerName,
    ])
  }

  /**
   * Simulates packet loss for a target container.
   * @param targetContainerName Name of the container to target.
   * @param lossPercent Packet loss percentage (0-100).
   * @param durationSec Duration of the fault in seconds.
   */
  public static async simulateLoss(
    targetContainerName: string,
    lossPercent: number,
    durationSec: number,
  ): Promise<void> {
    await this.runPumbaCommand([
      'netem',
      '--duration',
      `${durationSec}s`,
      'loss',
      '--percent',
      `${lossPercent}`,
      targetContainerName,
    ])
  }

  /**
   * Simulates bandwidth limitation for a target container.
   * @param targetContainerName Name of the container to target.
   * @param rate Bandwidth rate (e.g., "1mbit").
   * @param durationSec Duration of the fault in seconds.
   */
  public static async simulateBandwidth(
    targetContainerName: string,
    rate: string,
    durationSec: number,
  ): Promise<void> {
    await this.runPumbaCommand([
      'netem',
      '--duration',
      `${durationSec}s`,
      'rate',
      '--rate',
      rate,
      targetContainerName,
    ])
  }
}
