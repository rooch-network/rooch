// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

import { GenericContainer, AbstractStartedContainer, StartedTestContainer } from 'testcontainers'

// PumbaContainer: Configures and starts the Pumba container
export class PumbaContainer extends GenericContainer {
  constructor(image = 'gaiaadm/pumba:latest') {
    super(image)
    // Mount Docker socket to allow Pumba to control other containers
    this.withBindMounts([{ source: '/var/run/docker.sock', target: '/var/run/docker.sock' }])
      .withCommand(['sleep', 'infinity']) // Keep container running
      .withStartupTimeout(120_000) // Allow 120 seconds for startup
  }

  /**
   * Starts the Pumba container and returns a StartedPumbaContainer instance.
   * @returns Promise<StartedPumbaContainer>
   */
  public override async start(): Promise<StartedPumbaContainer> {
    const container = await super.start()
    return new StartedPumbaContainer(container)
  }
}

// StartedPumbaContainer: Provides methods to simulate network faults
export class StartedPumbaContainer extends AbstractStartedContainer {
  constructor(startedTestContainer: StartedTestContainer) {
    super(startedTestContainer)
  }

  /**
   * Executes a Pumba command inside the container.
   * @param command The Pumba command array to execute.
   * @throws Error if the command fails.
   */
  private async execPumbaCommand(command: string[]): Promise<void> {
    const result = await this.startedTestContainer.exec(command)
    if (result.exitCode !== 0) {
      throw new Error(`Pumba command failed with exit code ${result.exitCode}: ${result.output}`)
    }
  }

  /**
   * Simulates network delay for a target container.
   * @param targetContainerName Name of the container to target.
   * @param delayMs Delay in milliseconds.
   * @param durationSec Duration of the fault in seconds.
   */
  public async simulateDelay(
    targetContainerName: string,
    delayMs: number,
    durationSec: number,
  ): Promise<void> {
    await this.execPumbaCommand([
      'pumba',
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
  public async simulateLoss(
    targetContainerName: string,
    lossPercent: number,
    durationSec: number,
  ): Promise<void> {
    await this.execPumbaCommand([
      'pumba',
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
   * Simulates bandwidth limitation for a target container (optional extension).
   * @param targetContainerName Name of the container to target.
   * @param rate Bandwidth rate (e.g., "1mbit").
   * @param durationSec Duration of the fault in seconds.
   */
  public async simulateBandwidth(
    targetContainerName: string,
    rate: string,
    durationSec: number,
  ): Promise<void> {
    await this.execPumbaCommand([
      'pumba',
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
