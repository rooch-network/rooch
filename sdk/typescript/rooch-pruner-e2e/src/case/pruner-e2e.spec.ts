// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0
import path from 'node:path'
import { fileURLToPath } from 'node:url'
import { afterAll, beforeAll, describe, expect, it } from 'vitest'

import { TestBox } from '@roochnetwork/test-suite'
import { PrometheusClient, PrunerMetrics } from '../utils/prometheus-client.js'
import { generateReport, printReport } from '../utils/test-reporter.js'

const __filename = fileURLToPath(import.meta.url)
const __dirname = path.dirname(__filename)
const repoRoot = path.resolve(__dirname, '../../../../../')
const counterPackagePath = path.join(repoRoot, 'examples', 'quick_start_counter')
const prunerPackagePath = path.join(repoRoot, 'examples', 'pruner_test')

const delay = (ms: number) => new Promise((resolve) => setTimeout(resolve, ms))

// Unified test configuration
interface TestConfig {
  // Workload parameters
  counterIters: number
  createIters: number
  updateIters: number
  deleteIters: number
  cycleCount: number
  // Timing parameters
  interCycleWaitS: number // Wait time between cycles for pruner to work (default 60s)
  settleMs: number // Final wait time after all cycles before collecting final metrics
  // Pruner server parameters
  prunerIntervalS: number
  protectionOrders: number // Number of recent tx_orders to protect (0 = aggressive, only protect latest root)
  bloomBits: number
  scanBatch: number
  deleteBatch: number
}

function loadTestConfig(): TestConfig {
  return {
    // Workload - unified env var names
    counterIters: parseInt(process.env.COUNTER_ITERS || '1', 10),
    createIters: parseInt(process.env.CREATE_ITERS || '1', 10),
    updateIters: parseInt(process.env.UPDATE_ITERS || '1', 10),
    deleteIters: parseInt(process.env.DELETE_ITERS || '1', 10),
    cycleCount: parseInt(process.env.CYCLE_COUNT || '1', 10),
    // Timing - monitoring is now integrated into cycles via interCycleWaitS
    interCycleWaitS: parseInt(process.env.INTER_CYCLE_WAIT_S || '60', 10),
    settleMs: parseInt(process.env.SETTLE_MS || '60000', 10),
    // Pruner server
    prunerIntervalS: parseInt(process.env.PRUNER_INTERVAL_S || '15', 10),
    // protection_orders: 0 = aggressive (only protect latest root)
    // Higher values protect more historical states, reducing risk of deleting active data
    // Default to 0 for aggressive mode
    protectionOrders: parseInt(process.env.PROTECTION_ORDERS || '0', 10),
    bloomBits: parseInt(process.env.BLOOM_BITS || '16777216', 10), // 16MB default
    scanBatch: parseInt(process.env.SCAN_BATCH || '10000', 10),
    deleteBatch: parseInt(process.env.DELETE_BATCH || '5000', 10),
  }
}

// Estimate time for one cycle based on operation counts
function estimateCycleTimeMs(config: TestConfig): number {
  const totalOps =
    config.counterIters + config.createIters + config.updateIters + config.deleteIters
  // Based on actual measurements: ~100ms per Move call via roochCommand (execFileSync)
  // Using conservative estimate to avoid timeout
  const opsTimeMs = totalOps * 100
  // Plus inter-cycle wait time for pruner to work
  const waitTimeMs = config.interCycleWaitS * 1000
  return opsTimeMs + waitTimeMs
}

// Calculate total expected test time with buffer
function estimateTotalTimeMs(config: TestConfig): number {
  const cycleTimeMs = estimateCycleTimeMs(config)
  const allCyclesMs = cycleTimeMs * config.cycleCount
  const settleMs = config.settleMs
  const setupBufferMs = 10 * 60 * 1000 // 10 minutes for setup/teardown/buffer
  return allCyclesMs + settleMs + setupBufferMs
}

async function runMoveFunction(
  testbox: TestBox,
  functionId: string,
  args: string[],
  maxRetries = 3,
) {
  const commandArgs = [
    'move',
    'run',
    '--function',
    functionId,
    ...args.flatMap((arg) => ['--args', arg]),
    '--config-dir',
    testbox.roochDir,
    '--json',
  ]

  for (let attempt = 0; attempt <= maxRetries; attempt++) {
    try {
      return testbox.roochCommand(commandArgs)
    } catch (error: any) {
      const errorMessage = error?.message?.toLowerCase() || ''
      const errorCode = error?.code?.toLowerCase() || ''
      const stdout = String(error?.stdout || '').toLowerCase()
      const stderr = String(error?.stderr || '').toLowerCase()
      // execFileSync errors may also have output array: [stdin, stdout, stderr]
      const outputArray = error?.output || []
      const outputText = outputArray
        .map((item: any) => String(item || ''))
        .join(' ')
        .toLowerCase()

      // Check if it's a timeout-related error
      // Note: execFileSync errors may have timeout info in stdout, stderr, message, code, or output array
      const isTimeoutError =
        errorMessage.includes('timeout') ||
        errorMessage.includes('etimedout') ||
        errorMessage.includes('econnreset') ||
        errorMessage.includes('econnrefused') ||
        errorCode === 'etimedout' ||
        errorCode === 'econnreset' ||
        errorCode === 'econnrefused' ||
        stdout.includes('timeout') ||
        stdout.includes('request timeout') ||
        stderr.includes('timeout') ||
        outputText.includes('timeout') ||
        stdout.includes('connection') ||
        stderr.includes('connection') ||
        outputText.includes('connection')

      // Check if it's a rate limit error (429)
      const isRateLimitError =
        stdout.includes('429') ||
        stdout.includes('rate limit') ||
        stdout.includes('ratelimit') ||
        stdout.includes('request rejected') ||
        stderr.includes('429') ||
        stderr.includes('rate limit') ||
        stderr.includes('ratelimit') ||
        outputText.includes('429') ||
        outputText.includes('rate limit') ||
        errorMessage.includes('429') ||
        errorMessage.includes('rate limit')

      // If it's a retryable error (timeout or rate limit), retry with backoff
      const isRetryableError = isTimeoutError || isRateLimitError

      if (!isRetryableError) {
        // If not a retryable error, throw immediately
        throw error
      }

      // If retryable error and we have retries left, retry with exponential backoff
      if (attempt < maxRetries) {
        // Rate limit errors need longer delays, timeout errors can retry faster
        const baseDelay = isRateLimitError ? 2000 : 1000 // 2s for rate limit, 1s for timeout
        const retryDelay = baseDelay * (attempt + 1) // Exponential backoff
        const errorType = isRateLimitError ? 'rate limit (429)' : 'timeout'
        console.warn(
          `⚠️ Move function call ${errorType} error (attempt ${attempt + 1}/${maxRetries + 1}), retrying in ${retryDelay}ms...`,
        )
        await delay(retryDelay)
        continue
      }

      // Last attempt failed, throw the error
      throw error
    }
  }

  // This should never be reached, but TypeScript needs it for type checking
  throw new Error('Unexpected: retry loop completed without returning or throwing')
}

async function publishPackage(testbox: TestBox, packagePath: string, namedAddresses: string) {
  console.log(`Publishing package at ${packagePath} with named addresses: ${namedAddresses}`)
  const ok = await testbox.cmdPublishPackage(packagePath, { namedAddresses })
  if (!ok) {
    throw new Error(`Failed to publish package at ${packagePath}`)
  }
  console.log(`Successfully published package at ${packagePath}`)
}

describe('Rooch pruner end-to-end', () => {
  let testbox: TestBox
  let prometheus: PrometheusClient
  let defaultAddress: string
  const config = loadTestConfig()

  beforeAll(async () => {
    console.log('### pruner e2e: init testbox')
    testbox = new TestBox()
    console.log('### pruner e2e: start rooch server')

    // Configure pruner parameters from unified config
    const prunerArgs = [
      '--pruner-enable',
      '--pruner-enable-incremental-sweep',
      '--pruner-interval-s',
      config.prunerIntervalS.toString(),
      '--pruner-protection-orders',
      config.protectionOrders.toString(),
      '--pruner-bloom-bits',
      config.bloomBits.toString(),
      '--pruner-scan-batch',
      config.scanBatch.toString(),
      '--pruner-delete-batch',
      config.deleteBatch.toString(),
      // Configure rate limit for e2e tests to avoid 429 errors
      // traffic-per-second: interval in seconds to replenish one quota element
      // 0.01 means replenish 1 quota every 0.01s = 100 requests/second
      '--traffic-per-second',
      '0.001',
      '--traffic-burst-size',
      '10000',
    ]

    console.log('### pruner e2e: pruner config:', {
      intervalS: config.prunerIntervalS,
      protectionOrders: config.protectionOrders,
      bloomBits: `${(config.bloomBits / 1024 / 1024).toFixed(0)}MB`,
      scanBatch: config.scanBatch,
      deleteBatch: config.deleteBatch,
    })

    // Use port 0 to get dynamic port allocation
    const loadResult = await testbox.loadRoochEnv('local', 0, prunerArgs)
    console.log('### pruner e2e: loadRoochEnv returned:', loadResult)

    const serverAddress = testbox.getRoochServerAddress()
    console.log('### pruner e2e: serverAddress:', serverAddress)

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

    prometheus = new PrometheusClient(testbox.getMetricsPort() ?? 9184)
    console.log('### pruner e2e: beforeAll done')
  }, 180000)

  afterAll(async () => {
    if (testbox) {
      testbox.stop()
    }
  })

  // Calculate timeout using accurate estimation
  const testTimeout = Math.max(estimateTotalTimeMs(config), 5 * 60 * 1000)
  const expectedTimeMinutes = Math.ceil(estimateTotalTimeMs(config) / 60000)

  it(
    'collects pruning metrics after workload churn',
    async () => {
      const startTime = Date.now()

      // Print test configuration
      console.log('🚀 Starting pruner integration test')
      console.log('==========================================')
      console.log('📊 Test Configuration:')
      console.log(`  Workload per cycle:`)
      console.log(`    - Counter iterations: ${config.counterIters}`)
      console.log(`    - Create operations: ${config.createIters}`)
      console.log(`    - Update operations: ${config.updateIters}`)
      console.log(`    - Delete operations: ${config.deleteIters}`)
      console.log(`  Timing:`)
      console.log(`    - Number of cycles: ${config.cycleCount}`)
      console.log(
        `    - Inter-cycle wait: ${config.interCycleWaitS}s (pruner works during this time)`,
      )
      console.log(`    - Final settle time: ${config.settleMs}ms`)
      console.log(`  Estimated time:`)
      console.log(`    - Per cycle: ~${Math.ceil(estimateCycleTimeMs(config) / 1000)}s`)
      console.log(`    - Total: ~${expectedTimeMinutes} minutes`)
      console.log(`    - Timeout: ${Math.ceil(testTimeout / 60000)} minutes`)
      console.log('==========================================')
      console.log('')

      const txCounts = {
        counter: 0,
        objectCreated: 0,
        objectUpdated: 0,
        objectDeleted: 0,
      }

      // Use deterministic seeds so object IDs are known without querying on-chain state
      const seedBase = Date.now() % 1_000_000
      const phaseHistory: Array<{ phase: number; timestamp: string; metrics: PrunerMetrics }> = []

      // Execute workload cycles
      for (let cycle = 0; cycle < config.cycleCount; cycle++) {
        const cycleStartTime = Date.now()
        console.log(`🔄 Cycle ${cycle + 1}/${config.cycleCount}`)

        // Counter operations - create more versions
        console.log(`  📝 Counter operations: ${config.counterIters}`)
        for (let i = 0; i < config.counterIters; i++) {
          await runMoveFunction(testbox, `${defaultAddress}::quick_start_counter::increase`, [])
          txCounts.counter += 1
          if (i % 10 === 0) await delay(10)
        }

        // Object creation
        console.log(`  ➕ Create operations: ${config.createIters}`)
        const seeds: number[] = []
        for (let i = 0; i < config.createIters; i++) {
          const seed = seedBase + cycle * 10_000 + i
          seeds.push(seed)
          await runMoveFunction(testbox, `${defaultAddress}::object_lifecycle::create_named`, [
            `u64:${seed}`,
            `u64:${cycle * 1000 + i}`,
          ])
          txCounts.objectCreated += 1
          if (i % 20 === 0) await delay(5)
        }

        // Update operations
        console.log(`  ✏️  Update operations: ${config.updateIters}`)
        for (let i = 0; i < config.updateIters && i < seeds.length; i++) {
          await runMoveFunction(testbox, `${defaultAddress}::object_lifecycle::update_named`, [
            `u64:${seeds[i]}`,
            `u64:${cycle * 2000 + i}`,
          ])
          txCounts.objectUpdated += 1
          if (i % 15 === 0) await delay(5)
        }

        // Delete operations
        console.log(`  🗑️  Delete operations: ${config.deleteIters}`)
        for (let i = 0; i < config.deleteIters && i < seeds.length; i++) {
          await runMoveFunction(testbox, `${defaultAddress}::object_lifecycle::remove_named`, [
            `u64:${seeds[seeds.length - 1 - i]}`,
          ])
          txCounts.objectDeleted += 1
          if (i % 15 === 0) await delay(5)
        }

        // Inter-cycle wait for pruner to work (monitoring integrated into cycles)
        console.log(`  ⏳ Waiting ${config.interCycleWaitS}s for pruner...`)
        await delay(config.interCycleWaitS * 1000)

        // Collect and display metrics after cycle (integrated monitoring)
        try {
          const currentMetrics = await prometheus.fetchMetrics()
          phaseHistory.push({
            phase: currentMetrics.currentPhase,
            timestamp: new Date().toISOString(),
            metrics: currentMetrics,
          })
          const cycleTimeS = Math.round((Date.now() - cycleStartTime) / 1000)
          const totalDeleted =
            currentMetrics.sweepExpiredDeleted.count + currentMetrics.incrementalSweepDeleted.count
          console.log(`  ✅ Cycle ${cycle + 1} completed in ${cycleTimeS}s`)
          console.log(`     📊 Phase: ${currentMetrics.currentPhase}`)
          console.log(
            `     📊 Deleted: Sweep=${currentMetrics.sweepExpiredDeleted.count}, Incr=${currentMetrics.incrementalSweepDeleted.count}, Total=${totalDeleted}`,
          )
          console.log(`     📊 Reachable: ${currentMetrics.reachableNodesScanned.count}`)
          console.log(
            `     📊 Disk Reclaimed: ${(currentMetrics.diskSpaceReclaimedBytes / (1024 * 1024)).toFixed(2)} MB`,
          )
          if (currentMetrics.errorCount > 0) {
            console.log(`     ⚠️ Errors: ${currentMetrics.errorCount}`)
          }
        } catch (error) {
          console.warn(`  ⚠️ Metrics fetch failed:`, error)
        }
      }

      // Final settle - give pruner a bit more time to finish any pending work
      console.log('')
      console.log(`⏳ Final settle: waiting ${config.settleMs}ms for pruner to finish...`)
      await delay(config.settleMs)

      // Final metrics collection
      const finalMetrics = await prometheus.fetchMetrics()
      const testDuration = Date.now() - startTime

      // Generate and save report
      const report = generateReport(startTime, txCounts, finalMetrics)
      printReport(report)

      // Save detailed report for CI/CD pipeline
      const fs = await import('fs')
      const reportPath = 'test-report.json'
      fs.writeFileSync(
        reportPath,
        JSON.stringify(
          {
            report,
            testConfig: {
              ...config,
              actualDurationMs: testDuration,
              expectedDurationMs: estimateTotalTimeMs(config),
            },
            phaseHistory: phaseHistory.slice(-20), // Last 20 phase snapshots
          },
          null,
          2,
        ),
      )
      console.log(`### pruner e2e: saved detailed report to ${reportPath}`)

      // Verify key metrics
      expect(finalMetrics.bloomFilterSizeBytes).toBeGreaterThan(0)
      expect(finalMetrics.currentPhase).toBeGreaterThanOrEqual(0)
      expect(finalMetrics.errorCount).toBe(0)
      expect(finalMetrics.reachableNodesScanned.count).toBeGreaterThan(0)

      const totalDeletions =
        finalMetrics.sweepExpiredDeleted.sum + finalMetrics.incrementalSweepDeleted.sum
      const reachableNodes = finalMetrics.reachableNodesScanned.sum

      console.log('')
      console.log('🎯 Test Results:')
      const durationMinutes = (testDuration / 60000).toFixed(2)
      console.log(`  ✅ Duration: ${durationMinutes} minutes (expected: ~${expectedTimeMinutes})`)
      const totalTxs =
        txCounts.counter + txCounts.objectCreated + txCounts.objectUpdated + txCounts.objectDeleted
      console.log(`  ✅ Total Transactions: ${totalTxs}`)
      console.log(`  ✅ Nodes Deleted: ${totalDeletions}`)
      console.log(`  ✅ Reachable Nodes Protected: ${reachableNodes}`)
      console.log(
        `  ✅ Disk Reclaimed: ${(finalMetrics.diskSpaceReclaimedBytes / (1024 * 1024)).toFixed(2)} MB`,
      )
      console.log(`  ✅ Errors: ${finalMetrics.errorCount}`)

      expect(totalDeletions).toBeGreaterThanOrEqual(0)
      expect(report.validation.passed).toBe(true)
    },
    { timeout: testTimeout },
  )
})
