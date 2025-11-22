import path from 'node:path'
import { fileURLToPath } from 'node:url'
import { afterAll, beforeAll, describe, expect, it } from 'vitest'

import { TestBox } from '../testbox.js'
import { PrometheusClient } from '../utils/prometheus-client.js'
import { generateReport, printReport } from '../utils/test-reporter.js'

const __filename = fileURLToPath(import.meta.url)
const __dirname = path.dirname(__filename)
const repoRoot = path.resolve(__dirname, '../../../../..')
const counterPackagePath = path.join(repoRoot, 'examples', 'quick_start_counter')
const prunerPackagePath = path.join(repoRoot, 'examples', 'pruner_test')

const delay = (ms: number) => new Promise((resolve) => setTimeout(resolve, ms))

function runMoveFunction(testbox: TestBox, functionId: string, args: string[]) {
  testbox.roochCommand([
    'move',
    'run',
    '--function',
    functionId,
    ...args.flatMap((arg) => ['--args', arg]),
    '--config-dir',
    testbox.roochDir,
    '--json',
  ])
}

async function publishPackage(testbox: TestBox, packagePath: string, namedAddresses: string) {
  const ok = await testbox.cmdPublishPackage(packagePath, { namedAddresses })
  if (!ok) {
    throw new Error(`Failed to publish package at ${packagePath}`)
  }
}

async function publishPackageViaClient(testbox: TestBox, packagePath: string) {
  const address = await testbox.defaultCmdAddress()
  const client = testbox.getClient()

  // Build package
  testbox.roochCommand([
    'move',
    'build',
    '-p', packagePath,
    '--named-addresses', `default=${address},std=0x1,moveos_std=0x2,rooch_framework=0x3`,
    '--install-dir', testbox.roochDir,
    '--json'
  ])

  // Read built package
  const fs = await import('fs')
  const path = await import('path')
  const packageBytes = fs.readFileSync(path.join(testbox.roochDir, 'package.rpd'))

  // Publish via client
  const tx = new Transaction()
  tx.callFunction({
    target: '0x2::module_store::publish_package_entry',
    args: [Args.vec('u8', Array.from(packageBytes))],
  })

  const result = await client.signAndExecuteTransaction({
    transaction: tx,
    signer: await testbox.getDefaultKeyPair(),
  })

  if (result.execution_info.status.type !== 'executed') {
    throw new Error(`Failed to publish package at ${packagePath}: ${JSON.stringify(result.execution_info.status)}`)
  }

  console.log(`✅ Successfully published package at ${packagePath}`)
}

describe('Rooch pruner end-to-end', () => {
  let testbox: TestBox
  let prometheus: PrometheusClient
  let defaultAddress: string

  beforeAll(async () => {
    console.log('### pruner e2e: init testbox')
    testbox = new TestBox()
    console.log('### pruner e2e: start rooch server')
    // Use port 0 to get dynamic port allocation
    await testbox.loadRoochEnv('local', 0, [
      '--pruner-enable',
      '--pruner-interval-s',
      '15',
      '--pruner-window-days',
      '0',
      '--pruner-enable-incremental-sweep',
      '--pruner-bloom-bits',
      '16777216', // 16MB bloom filter to keep memory reasonable in tests
    ])

    // Configure RPC URL for rooch CLI after server starts
    const serverAddress = testbox.getRoochServerAddress()
    if (serverAddress) {
      console.log('### pruner e2e: configuring RPC URL:', serverAddress)
      testbox.roochCommand([
        'env',
        'add',
        '--config-dir',
        testbox.roochDir,
        '--alias',
        'local',
        '--rpc',
        `http://${serverAddress}`,
      ])
      testbox.roochCommand(['env', 'switch', '--config-dir', testbox.roochDir, '--alias', 'local'])
    }

    console.log('### pruner e2e: fetch default address')
    defaultAddress = await testbox.defaultCmdAddress()
    console.log('### pruner e2e: default address:', defaultAddress)

    // Wait for server to be fully ready before publishing packages
    console.log('### pruner e2e: waiting 10s for server to be fully ready...')
    await delay(10000)

    console.log('### pruner e2e: publish counter package')
    await publishPackage(
      testbox,
      counterPackagePath,
      'quick_start_counter=default,std=0x1,moveos_std=0x2,rooch_framework=0x3',
    )
    console.log('### pruner e2e: publish pruner package')
    await publishPackage(
      testbox,
      prunerPackagePath,
      'pruner_test=default,std=0x1,moveos_std=0x2,rooch_framework=0x3',
    )

    // Wait extra time for server to be fully ready
    console.log('### pruner e2e: waiting for server to be fully ready')
    await delay(10000)

    prometheus = new PrometheusClient(testbox.getMetricsPort() ?? 9184)
    console.log('### pruner e2e: beforeAll done')
  }, 180000)

  afterAll(async () => {
    testbox.stop()
  })

  it(
    'collects pruning metrics after workload churn',
    async () => {
      const startTime = Date.now()
      // Allow load shedding via env to keep memory/time bounded in CI or local runs
      const counterIters = parseInt(process.env.PRUNER_COUNTER_ITERS || '1', 10)
      const createIters = parseInt(process.env.PRUNER_CREATE_ITERS || '1', 10)
      const updateIters = parseInt(process.env.PRUNER_UPDATE_ITERS || '1', 10)
      const deleteIters = parseInt(process.env.PRUNER_DELETE_ITERS || '1', 10)
      const settleMs = parseInt(process.env.PRUNER_SETTLE_MS || '15000', 10)
      const txCounts = {
        counter: 0,
        objectCreated: 0,
        objectUpdated: 0,
        objectDeleted: 0,
      }

      // Counter churn to generate stale versions
      for (let i = 0; i < counterIters; i++) {
        runMoveFunction(
          testbox,
          `${defaultAddress}::quick_start_counter::increase`,
          [], // no args
        )
        txCounts.counter += 1
        await delay(50)
      }

      // Object lifecycle workload
      for (let i = 0; i < createIters; i++) {
        runMoveFunction(
          testbox,
          `${defaultAddress}::object_lifecycle::create_object`,
          [`u64:${i}`, `u64:${i}`],
        )
        txCounts.objectCreated += 1
        await delay(30)
      }

      // Note: Skip update/remove operations for now as they require object IDs from previous operations
      console.log('### pruner e2e: skipping update/delete operations for simplicity')

      // Allow at least one full pruning cycle
      await delay(settleMs)

      const prunerMetrics = await prometheus.fetchMetrics()
      const report = generateReport(startTime, txCounts, prunerMetrics)
      printReport(report)

      expect(prunerMetrics.bloomFilterSizeBytes).toBeGreaterThan(0)
      expect(prunerMetrics.currentPhase).toBeGreaterThanOrEqual(0)
      expect(prunerMetrics.errorCount).toBe(0)
      expect(
        prunerMetrics.sweepExpiredDeleted.sum + prunerMetrics.incrementalSweepDeleted.sum,
      ).toBeGreaterThan(0)
      expect(report.validation.passed).toBe(true)
    },
    240000,
  )
})
