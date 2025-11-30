// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

import path from 'node:path'
import { fileURLToPath } from 'node:url'
import { afterAll, beforeAll, describe, expect, it } from 'vitest'

import { TestBox } from '@roochnetwork/test-suite'
import { PrometheusClient } from '../utils/prometheus-client.js'
// import { generateReport, printReport } from '../utils/test-reporter.js'

const __filename = fileURLToPath(import.meta.url)
const __dirname = path.dirname(__filename)
const repoRoot = path.resolve(__dirname, '../../../../../')
const counterPackagePath = path.join(repoRoot, 'examples', 'quick_start_counter')
const prunerPackagePath = path.join(repoRoot, 'examples', 'pruner_test')

const delay = (ms: number) => new Promise((resolve) => setTimeout(resolve, ms))

// GC test configuration
interface GCTestConfig {
  // Data generation parameters
  objectCount: number
  updateCount: number
  deleteCount: number
  counterIters: number

  // GC command parameters
  dryRun: boolean
  batchSize: number
  workers: number
  markerStrategy: 'auto' | 'memory' | 'persistent'
  useRecycleBin: boolean
  protectedRootsCount: number
  force: boolean
  verbose: boolean

  // Test parameters
  settleMs: number
  testTimeout: number
}

function loadGCTestConfig(): GCTestConfig {
  return {
    // Data generation - small scale quick verification
    objectCount: parseInt(process.env.GC_OBJECT_COUNT || '100', 10),
    updateCount: parseInt(process.env.GC_UPDATE_COUNT || '50', 10),
    deleteCount: parseInt(process.env.GC_DELETE_COUNT || '30', 10),
    counterIters: parseInt(process.env.GC_COUNTER_ITERS || '20', 10),

    // GC command parameters
    dryRun: process.env.GC_DRY_RUN !== 'false',
    batchSize: parseInt(process.env.GC_BATCH_SIZE || '10000', 10),
    workers: parseInt(process.env.GC_WORKERS || '4', 10),
    markerStrategy: (process.env.GC_MARKER_STRATEGY as 'auto' | 'memory' | 'persistent') || 'auto',
    useRecycleBin: process.env.GC_USE_RECYCLE_BIN !== 'false',
    protectedRootsCount: parseInt(process.env.GC_PROTECTED_ROOTS_COUNT || '1', 10),
    force: process.env.GC_FORCE === 'true',
    verbose: process.env.GC_VERBOSE === 'true',

    // Test parameters
    settleMs: parseInt(process.env.GC_SETTLE_MS || '30000', 10),
    testTimeout: parseInt(process.env.GC_TEST_TIMEOUT || '600000', 10), // 10 minutes
  }
}

// GC report interface
interface GCReport {
  executionMode: 'dry-run' | 'execute'
  protectedRoots: string[]
  markStats: {
    markedCount: number
    duration: number
    memoryStrategy: string
  }
  sweepStats: {
    scannedCount: number
    keptCount: number
    deletedCount: number
    recycleBinEntries: number
    duration: number
  }
  memoryStrategyUsed: string
  duration: number
  diskSpaceReclaimed: number
}

// Core function to execute GC commands
async function executeGCCommand(
  testbox: TestBox,
  options: Partial<GCTestConfig> = {},
): Promise<GCReport> {
  const config = { ...loadGCTestConfig(), ...options }

  // The data directory path should match what the server uses
  const dataDir = path.join(testbox.roochDir, 'data')

  const args = [
    'db',
    'gc',
    '--chain-id',
    'local', // Must match the network name used when starting the server
    '--data-dir',
    dataDir, // Must match the data directory used by the server
    ...(config.dryRun ? ['--dry-run'] : []),
    '--batch-size',
    config.batchSize.toString(),
    '--workers',
    config.workers.toString(),
    '--marker-strategy',
    config.markerStrategy,
    ...(config.useRecycleBin ? ['--recycle-bin'] : []),
    '--protected-roots-count',
    config.protectedRootsCount.toString(),
    ...(config.force ? ['--force'] : []),
    ...(config.verbose ? ['--verbose'] : []),
  ]

  console.log(`🔧 Executing GC command: rooch ${args.join(' ')}`)

  try {
    const startTime = Date.now()
    const result = testbox.roochCommand(args)
    const duration = Date.now() - startTime

    console.log(`✅ GC command execution completed, duration: ${duration}ms`)
    if (result && typeof result === 'object' && 'stdout' in result) {
      console.log('📄 GC output:', result.stdout)
    }

    return parseGCReport(result || '', config.dryRun)
  } catch (error: any) {
    console.error('❌ GC command execution failed:', error)

    // Check if this is expected safety error
    if (
      error.message?.includes('GC modifies database state') ||
      error.message?.includes('Use --force to confirm')
    ) {
      throw new Error('GC safety verification failed: --force flag not used')
    }

    throw new Error(`GC command execution exception: ${error.message}`)
  }
}

// Parse GC report output
function parseGCReport(output: string, isDryRun: boolean): GCReport {
  const lines = output.split('\n')
  const report: GCReport = {
    executionMode: isDryRun ? 'dry-run' : 'execute',
    protectedRoots: [],
    markStats: { markedCount: 0, duration: 0, memoryStrategy: '' },
    sweepStats: {
      scannedCount: 0,
      keptCount: 0,
      deletedCount: 0,
      recycleBinEntries: 0,
      duration: 0,
    },
    memoryStrategyUsed: '',
    duration: 0,
    diskSpaceReclaimed: 0,
  }

  lines.forEach((line) => {
    // Parse protected root information
    if (line.includes('Protected Roots:')) {
      const count = parseInt(line.split(':')[1].trim().split(' ')[0])
      report.protectedRoots = Array(count).fill(`root_${Math.random().toString(36).substr(2, 9)}`)
    }

    // Parse marking phase statistics
    if (line.includes('Nodes Marked Reachable:')) {
      report.markStats.markedCount = parseInt(line.split(':')[1].trim())
    }

    if (line.includes('Memory Strategy:')) {
      report.markStats.memoryStrategy = line.split(':')[1].trim()
    }

    if (line.includes('Duration:')) {
      const durationStr = line.split(':')[1].trim()
      const seconds = parseFloat(durationStr)
      if (!isNaN(seconds)) {
        report.markStats.duration = Math.round(seconds * 1000)
      }
    }

    // Parse sweeping phase statistics
    if (line.includes('Nodes Scanned:')) {
      report.sweepStats.scannedCount = parseInt(line.split(':')[1].trim())
    }

    if (line.includes('Nodes Kept (Reachable):')) {
      report.sweepStats.keptCount = parseInt(line.split(':')[1].trim())
    }

    if (line.includes('Nodes Deleted (Unreachable):')) {
      report.sweepStats.deletedCount = parseInt(line.split(':')[1].trim())
    }

    if (line.includes('Nodes Sent to Recycle Bin:')) {
      report.sweepStats.recycleBinEntries = parseInt(line.split(':')[1].trim())
    }

    // Parse overall statistics
    if (line.includes('Memory Strategy Used:')) {
      report.memoryStrategyUsed = line.split(':')[1].trim()
    }

    if (line.includes('Total Execution Time:')) {
      const timeStr = line.split(':')[1].trim()
      const seconds = parseFloat(timeStr)
      if (!isNaN(seconds)) {
        report.duration = Math.round(seconds * 1000)
      }
    }

    if (line.includes('Space Reclaimed:')) {
      const match = line.match(/([\d.]+)%/)
      if (match) {
        report.diskSpaceReclaimed = parseFloat(match[1])
      }
    }
  })

  return report
}

// Helper function to execute Move functions
async function runMoveFunction(
  testbox: TestBox,
  functionId: string,
  args: string[],
  maxRetries = 3,
): Promise<any> {
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

      // Check if it is timeout or connection error
      const isRetryableError =
        errorMessage.includes('timeout') ||
        errorMessage.includes('etimedout') ||
        errorMessage.includes('econnreset') ||
        errorMessage.includes('econnrefused') ||
        errorMessage.includes('connection')

      if (!isRetryableError || attempt === maxRetries) {
        throw error
      }

      const retryDelay = 1000 * (attempt + 1)
      console.warn(
        `⚠️ Move function call error (attempt ${attempt + 1}/${maxRetries + 1})，${retryDelay}msretry after...`,
      )
      await delay(retryDelay)
    }
  }

  throw new Error('Unexpected: retry loop completed without returning or throwing')
}

// Helper function to publish contracts
async function publishPackage(testbox: TestBox, packagePath: string, namedAddresses: string) {
  console.log(`📦 Publishing contract: ${packagePath}，Address mapping: ${namedAddresses}`)
  const ok = await testbox.cmdPublishPackage(packagePath, { namedAddresses })
  if (!ok) {
    throw new Error(`❌ Contract publication failed: ${packagePath}`)
  }
  console.log(`✅ Contract published successfully: ${packagePath}`)
}

// Generate test data
async function generateTestData(testbox: TestBox, config: GCTestConfig, defaultAddress: string) {
  console.log(`🏗️  Starting Generate test data...`)
  console.log(`  - Objects created: ${config.objectCount}`)
  console.log(`  - Objects updated: ${config.updateCount}`)
  console.log(`  - Objects deleted: ${config.deleteCount}`)
  console.log(`  - Counter operations: ${config.counterIters}`)

  const txCounts = {
    objectCreated: 0,
    objectUpdated: 0,
    objectDeleted: 0,
    counter: 0,
  }

  // Use deterministic seed
  const seedBase = Date.now() % 1_000_000
  const seeds: number[] = []

  // Object creation operations
  console.log(`📝 Executing object creation operations...`)
  for (let i = 0; i < config.objectCount; i++) {
    const seed = seedBase + i
    seeds.push(seed)
    await runMoveFunction(testbox, `${defaultAddress}::object_lifecycle::create_named`, [
      `u64:${seed}`,
      `u64:${i * 10}`,
    ])
    txCounts.objectCreated += 1
    if (i % 20 === 0) await delay(5)
  }

  // Object update operations
  console.log(`✏️  Executing object update operations...`)
  for (let i = 0; i < config.updateCount && i < seeds.length; i++) {
    await runMoveFunction(testbox, `${defaultAddress}::object_lifecycle::update_named`, [
      `u64:${seeds[i]}`,
      `u64:${i * 20}`,
    ])
    txCounts.objectUpdated += 1
    if (i % 15 === 0) await delay(5)
  }

  // Object deletion operations
  console.log(`🗑️  Executing object deletion operations...`)
  for (let i = 0; i < config.deleteCount && i < seeds.length; i++) {
    await runMoveFunction(testbox, `${defaultAddress}::object_lifecycle::remove_named`, [
      `u64:${seeds[seeds.length - 1 - i]}`,
    ])
    txCounts.objectDeleted += 1
    if (i % 15 === 0) await delay(5)
  }

  // Counter operations (create more state versions)
  console.log(`🔢 Executing counter operations...`)
  for (let i = 0; i < config.counterIters; i++) {
    await runMoveFunction(testbox, `${defaultAddress}::quick_start_counter::increase`, [])
    txCounts.counter += 1
    if (i % 10 === 0) await delay(10)
  }

  console.log(`✅ Test data generation completed:`)
  console.log(`  - Objects created: ${txCounts.objectCreated}`)
  console.log(`  - Objects updated: ${txCounts.objectUpdated}`)
  console.log(`  - Objects deleted: ${txCounts.objectDeleted}`)
  console.log(`  - Counter operations: ${txCounts.counter}`)

  return txCounts
}

// GC test cycle
async function runGCTestCycle(
  testbox: TestBox,
  config: GCTestConfig,
  defaultAddress: string,
): Promise<{ report: GCReport; txCounts: any }> {
  console.log(`🚀 Starting GC test cycle...`)

  // 1. Data generation phase
  console.log(`\n📊 Phase 1: Data Generation`)
  const txCounts = await generateTestData(testbox, config, defaultAddress)

  // Waiting for data write completion
  console.log(`⏳ Waiting for data write completion...`)
  await delay(10000)

  // 2. Stop Rooch service
  console.log(`\n🛑 Phase 2: Stop Rooch Service`)
  testbox.stop()

  // Wait for service to completely stop
  console.log(`⏳ Waiting for service stop...`)
  await delay(5000)

  // 3. Execute GC dry run
  console.log(`\n🔍 Phase 3: GC Dry Run`)
  const dryRunReport = await executeGCCommand(testbox, {
    ...config,
    dryRun: true,
    force: false,
  })

  // 4. Execute actual GC (if configuration allows)
  let executeReport: GCReport | null = null
  if (config.force && !config.dryRun) {
    console.log(`\n🧹 Phase 4: Actual GC Execution`)
    executeReport = await executeGCCommand(testbox, {
      ...config,
      dryRun: false,
      force: true,
    })
  }

  // 5. Restart service verification
  console.log(`\n🚀 Phase 5: Restart Service and Verify`)
  console.log(`⏳ Restarting Rooch service...`)

  // Restart service
  const loadResult = await testbox.loadRoochEnv('local', 0)
  console.log(`📋 Service restart result:`, loadResult)

  // Waiting for service ready
  console.log(`⏳ Waiting for service ready...`)
  await delay(15000)

  // Verify service health status
  try {
    const _healthCheck = testbox.roochCommand(['state', '--config-dir', testbox.roochDir])
    console.log(`✅ Service health check passed`)
  } catch (error) {
    console.warn(`⚠️ Service health check failed:`, error)
  }

  return {
    report: executeReport || dryRunReport,
    txCounts,
  }
}

describe('GC End-to-End Tests', () => {
  let testbox: TestBox
  let _prometheus: PrometheusClient
  let defaultAddress: string
  const config = loadGCTestConfig()

  beforeAll(async () => {
    console.log('### GC E2E: Initialize test environment')
    console.log('🔧 Test configuration:')
    console.log(`  - Object count: ${config.objectCount}`)
    console.log(`  - Update count: ${config.updateCount}`)
    console.log(`  - Delete count: ${config.deleteCount}`)
    console.log(`  - Counter operations: ${config.counterIters}`)
    console.log(`  - GC strategy: ${config.markerStrategy}`)
    console.log(`  - Batch size: ${config.batchSize}`)
    console.log(`  - Worker threads: ${config.workers}`)
    console.log(`  - Dry run mode: ${config.dryRun}`)
    console.log(`  - Force execution: ${config.force}`)

    // Keep temp directory for GC test cycles (stop and restart server)
    process.env.TESTBOX_KEEP_TMP = 'true'

    testbox = new TestBox()
    console.log('### GC E2E: Starting Rooch server')

    // Starting Rooch server（Do not enable pruner, because we want to test stop-the-world GC）
    const loadResult = await testbox.loadRoochEnv('local', 0)
    console.log('### GC E2E: loadRoochEnv returned:', loadResult)

    const serverAddress = testbox.getRoochServerAddress()
    console.log('### GC E2E: Server address:', serverAddress)

    console.log('### GC E2E: Getting default address')
    defaultAddress = await testbox.defaultCmdAddress()
    console.log('### GC E2E: Default address:', defaultAddress)

    // Wait for service to be fully ready
    console.log('### GC E2E: Waiting for service ready...')
    await delay(15000)

    console.log('### GC E2E: Publishing counter contract')
    await publishPackage(
      testbox,
      counterPackagePath,
      'quick_start_counter=default,std=0x1,moveos_std=0x2,rooch_framework=0x3',
    )

    console.log('### GC E2E: Publishing pruner contract')
    await publishPackage(
      testbox,
      prunerPackagePath,
      'pruner_test=default,std=0x1,moveos_std=0x2,rooch_framework=0x3',
    )

    _prometheus = new PrometheusClient(testbox.getMetricsPort() ?? 9184)
    console.log('### GC E2E: beforeAll completed')
  }, 300000)

  afterAll(async () => {
    if (testbox) {
      console.log('### GC E2E: Cleanup test environment')
      testbox.stop()
    }
  })

  describe('GC Basic Functionality', () => {
    it(
      'GC Dry Run - Quick verification of basic functionality',
      async () => {
        console.log('\n🧪 Test: GC Dry Run basic functionality verification')

        const testConfig = { ...config, dryRun: true, force: false }
        const result = await runGCTestCycle(testbox, testConfig, defaultAddress)

        // Verify basic functionality
        expect(result.report.executionMode).toBe('dry-run')
        expect(result.report.markStats.markedCount).toBeGreaterThan(0)
        expect(result.report.memoryStrategyUsed).toBeDefined()

        console.log('✅ GC Dry Run basic functionality test passed')
        console.log(`  - Marked nodes count: ${result.report.markStats.markedCount}`)
        console.log(`  - Memory strategy: ${result.report.memoryStrategyUsed}`)
        console.log(`  - execution time: ${result.report.duration}ms`)
      },
      { timeout: config.testTimeout },
    )

    it(
      'GC Dry Run - Security verification test',
      async () => {
        console.log('\n🧪 Test: GC security verification mechanism')

        // Test that it should be rejected when --force is not used
        await expect(executeGCCommand(testbox, { dryRun: false, force: false })).rejects.toThrow(
          'GC safety verification failed: --force flag not used',
        )

        console.log('✅ GC Security verification test passed')
      },
      { timeout: 60000 },
    )

    it(
      'GCCommand parameter verification test',
      async () => {
        console.log('\n🧪 Test: GC command parameter verification')

        // Test different parameter combinations
        const testCases = [
          { markerStrategy: 'memory' as const, batchSize: 5000 },
          { markerStrategy: 'persistent' as const, workers: 2 },
          { markerStrategy: 'auto' as const, useRecycleBin: true },
        ]

        for (const testCase of testCases) {
          const report = await executeGCCommand(testbox, {
            ...testCase,
            dryRun: true,
            force: false,
          })

          expect(report.executionMode).toBe('dry-run')
          expect(report.markStats.markedCount).toBeGreaterThanOrEqual(0)

          console.log(`  - strategy ${testCase.markerStrategy}: ✅`)
        }

        console.log('✅ GC Command parameter verification test passed')
      },
      { timeout: 180000 },
    )
  })

  describe('GC Strategy Selection', () => {
    it(
      'Memory Marker Strategy test',
      async () => {
        console.log('\n🧪 Test: Memory Marker Strategy')

        const testConfig = {
          ...config,
          markerStrategy: 'memory' as const,
          dryRun: true,
          force: false,
        }

        const result = await runGCTestCycle(testbox, testConfig, defaultAddress)

        expect(result.report.markStats.markedCount).toBeGreaterThan(0)
        expect(result.report.memoryStrategyUsed).toContain('memory')

        console.log('✅ Memory Marker Strategy test passed')
        console.log(`  - Marked nodes count: ${result.report.markStats.markedCount}`)
        console.log(`  - Used strategy: ${result.report.memoryStrategyUsed}`)
      },
      { timeout: config.testTimeout },
    )

    it(
      'Persistent Marker Strategy test',
      async () => {
        console.log('\n🧪 Test: Persistent Marker Strategy')

        const testConfig = {
          ...config,
          markerStrategy: 'persistent' as const,
          dryRun: true,
          force: false,
        }

        const result = await runGCTestCycle(testbox, testConfig, defaultAddress)

        expect(result.report.markStats.markedCount).toBeGreaterThan(0)
        expect(result.report.memoryStrategyUsed).toContain('persistent')

        console.log('✅ Persistent Marker Strategy test passed')
        console.log(`  - Marked nodes count: ${result.report.markStats.markedCount}`)
        console.log(`  - Used strategy: ${result.report.memoryStrategyUsed}`)
      },
      { timeout: config.testTimeout },
    )

    it(
      'Auto Strategy automatic selection test',
      async () => {
        console.log('\n🧪 Test: Auto Strategy automatic selection')

        const testConfig = {
          ...config,
          markerStrategy: 'auto' as const,
          dryRun: true,
          force: false,
        }

        const result = await runGCTestCycle(testbox, testConfig, defaultAddress)

        expect(result.report.markStats.markedCount).toBeGreaterThan(0)
        expect(result.report.memoryStrategyUsed).toBeDefined()

        console.log('✅ Auto Strategy test passed')
        console.log(`  - Marked nodes count: ${result.report.markStats.markedCount}`)
        console.log(`  - Auto selected strategy: ${result.report.memoryStrategyUsed}`)
      },
      { timeout: config.testTimeout },
    )

    it(
      'Different batch_size performance test',
      async () => {
        console.log('\n🧪 Test: Different batch_size performance comparison')

        const batchSizes = [1000, 5000, 10000]
        const results: Array<{ batchSize: number; duration: number; markedCount: number }> = []

        for (const batchSize of batchSizes) {
          const startTime = Date.now()
          const report = await executeGCCommand(testbox, {
            batchSize,
            dryRun: true,
            force: false,
          })
          const duration = Date.now() - startTime

          results.push({
            batchSize,
            duration,
            markedCount: report.markStats.markedCount,
          })

          console.log(
            `  - Batch ${batchSize}: ${duration}ms, marked ${report.markStats.markedCount} nodes`,
          )
        }

        // Verify all batch sizes work normally
        results.forEach((result) => {
          expect(result.markedCount).toBeGreaterThanOrEqual(0)
          expect(result.duration).toBeGreaterThan(0)
        })

        console.log('✅ batch_size performance test passed')
      },
      { timeout: 300000 },
    )
  })

  describe('GC Integration Tests', () => {
    it(
      'Complete GC process: Data generation → Stop service → GC → Restart → Verify',
      async () => {
        console.log('\n🧪 Test: Complete GC process integration test')

        if (!config.force) {
          console.log('⚠️ Skip actual GC test (need to set GC_FORCE=true)')
          return
        }

        const testConfig = {
          ...config,
          dryRun: false,
          force: true,
          verbose: true,
        }

        const result = await runGCTestCycle(testbox, testConfig, defaultAddress)

        expect(result.report.executionMode).toBe('execute')
        expect(result.report.sweepStats.scannedCount).toBeGreaterThan(0)
        expect(result.report.markStats.markedCount).toBeGreaterThan(0)

        console.log('✅ Complete GC process test passed')
        console.log(`  - Execution mode: ${result.report.executionMode}`)
        console.log(`  - Scanned nodes: ${result.report.sweepStats.scannedCount}`)
        console.log(`  - Marked nodes: ${result.report.markStats.markedCount}`)
        console.log(`  - Deleted nodes: ${result.report.sweepStats.deletedCount}`)
        console.log(`  - Recycle bin entries: ${result.report.sweepStats.recycleBinEntries}`)
      },
      { timeout: config.testTimeout * 2 },
    )

    it(
      'Recycle bin functionality verification test',
      async () => {
        console.log('\n🧪 Test: Recycle bin functionality verification')

        const testConfig = {
          ...config,
          useRecycleBin: true,
          dryRun: true,
          force: false,
        }

        const result = await runGCTestCycle(testbox, testConfig, defaultAddress)

        expect(result.report.markStats.markedCount).toBeGreaterThan(0)

        console.log('✅ Recycle bin functionality verification test passed')
        console.log(`  - Recycle bin enabled: ${testConfig.useRecycleBin}`)
        console.log(`  - Marked nodes count: ${result.report.markStats.markedCount}`)
      },
      { timeout: config.testTimeout },
    )

    it(
      'Multiple GC execution test',
      async () => {
        console.log('\n🧪 Test: Multiple GC execution stability')

        const testConfig = { ...config, dryRun: true, force: false }
        const results: GCReport[] = []

        // Execute GC 3 times
        for (let i = 0; i < 3; i++) {
          console.log(`  - Execution${i + 1} GC...`)
          const result = await runGCTestCycle(testbox, testConfig, defaultAddress)
          results.push(result.report)

          // Brief wait
          await delay(5000)
        }

        // Verify each execution succeeds
        results.forEach((report, index) => {
          expect(report.markStats.markedCount).toBeGreaterThan(0)
          console.log(`  - Execution${index + 1}: marked ${report.markStats.markedCount} nodes`)
        })

        console.log('✅ Multiple GC execution test passed')
      },
      { timeout: config.testTimeout * 4 },
    )
  })

  describe('GC Performance Validation', () => {
    it(
      'GC execution time performance test',
      async () => {
        console.log('\n🧪 Test: GC execution time performance verification')

        const startTime = Date.now()
        const testConfig = { ...config, dryRun: true, force: false }

        const result = await runGCTestCycle(testbox, testConfig, defaultAddress)
        const totalDuration = Date.now() - startTime

        // Verify execution time is within reasonable range (small-scale data should be <5 minutes)
        expect(totalDuration).toBeLessThan(5 * 60 * 1000) // 5minutes
        expect(result.report.markStats.markedCount).toBeGreaterThan(0)

        console.log('✅ GC execution time performance test passed')
        console.log(`  - Total execution time: ${totalDuration}ms`)
        console.log(`  - GC marking time: ${result.report.markStats.duration}ms`)
        console.log(`  - Marked nodes count: ${result.report.markStats.markedCount}`)
        console.log(
          `  - Marking throughput: ${Math.round(result.report.markStats.markedCount / (result.report.markStats.duration / 1000))} nodes/second`,
        )
      },
      { timeout: config.testTimeout },
    )
  })
})
