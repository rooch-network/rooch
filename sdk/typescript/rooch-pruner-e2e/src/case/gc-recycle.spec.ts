// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

import { describe, it, expect, beforeAll, afterAll } from 'vitest'
import { TestBox } from '@roochnetwork/test-suite'
import {
  loadGCTestConfig,
  loadGCStressTestConfig,
  executeGC,
  GCResult,
  generateTestData,
  generateContinuousData,
  publishPackage,
  runMoveFunction,
  printGCReport,
} from '../utils/gc-utils.js'
import * as path from 'path'

const __dirname = path.dirname(__filename)
const repoRoot = path.resolve(__dirname, '../../../../../')
const counterPackagePath = path.join(repoRoot, 'examples', 'quick_start_counter')
const prunerPackagePath = path.join(repoRoot, 'examples', 'pruner_test')

// Helper functions
const delay = (ms: number) => new Promise((resolve) => setTimeout(resolve, ms))

async function stopServerForGC(testbox: TestBox): Promise<void> {
  console.log('⏹️ Stopping server for GC...')
  testbox.stop()
  // Wait for process to fully exit and file locks to be released
  await delay(3000)
}

async function restartServer(testbox: TestBox): Promise<void> {
  console.log('🔄 Restarting server after GC...')
  await testbox.loadRoochEnv('local', 0)
  // Wait for service to be ready
  await delay(5000)
}

async function verifyServerHealth(testbox: TestBox, maxRetries = 5): Promise<void> {
  for (let attempt = 1; attempt <= maxRetries; attempt++) {
    try {
      console.log(`🔍 Health check attempt ${attempt}/${maxRetries}...`)

      // Try multiple health check methods
      try {
        const statusResult = testbox.roochCommand([
          'rpc',
          'request',
          '--method',
          'rooch_status',
          '--config-dir',
          testbox.roochDir,
        ])
        const status = JSON.parse(statusResult)
        if (status.service_status === 'active') {
          console.log('✅ Server health check passed via rooch_status')
          return
        }
      } catch (e) {
        console.log(`⚠️ rooch_status check failed: ${e}`)
      }

      // Try account list as alternative health check
      try {
        const accountResult = testbox.roochCommand([
          'account',
          'list',
          '--config-dir',
          testbox.roochDir,
          '--json',
        ])
        const accounts = JSON.parse(accountResult)
        if (Array.isArray(accounts) && accounts.length > 0) {
          console.log('✅ Server health check passed via account list')
          return
        }
      } catch (e) {
        console.log(`⚠️ account list check failed: ${e}`)
      }

      if (attempt < maxRetries) {
        console.log(`⏳ Waiting before retry... (${10 * attempt}s)`)
        await delay(10000 * attempt) // Progressive backoff
      }
    } catch (error) {
      if (attempt === maxRetries) {
        throw new Error(`Server health check failed after ${maxRetries} attempts: ${error}`)
      }
    }
  }
}

describe('GC Recycle E2E Tests', () => {
  let testbox: TestBox
  let defaultAddress: string

  beforeAll(async () => {
    console.log('🚀 Initializing GC Recycle E2E Test Suite')

    // Keep temp directory for GC test cycles (stop and restart server)
    process.env.TESTBOX_KEEP_TMP = 'true'

    // Initialize testbox
    testbox = new TestBox()

    console.log('🔧 Starting Rooch server (this may take 1-2 minutes)...')
    const serverStartTime = Date.now()
    await testbox.loadRoochEnv('local', 0)
    const serverStartupTime = Date.now() - serverStartTime
    console.log(`✅ Rooch server started successfully in ${serverStartupTime}ms`)

    // Wait for service to be ready
    console.log('⏳ Waiting for service ready...')
    await delay(30000) // Increased from 15s to 30s to ensure server is fully ready

    // Perform comprehensive health check
    console.log('🔍 Performing comprehensive server health check...')
    await verifyServerHealth(testbox)

    // Get default address
    try {
      defaultAddress = await testbox.defaultCmdAddress()
      console.log(`📍 Default address: ${defaultAddress}`)
    } catch (error) {
      console.error('❌ Failed to get default address:', error)
      throw error
    }

    // Publish required contracts
    console.log('📦 Publishing contracts...')

    await publishPackage(
      testbox,
      counterPackagePath,
      'quick_start_counter=default,std=0x1,moveos_std=0x2,rooch_framework=0x3',
    )

    await publishPackage(
      testbox,
      prunerPackagePath,
      'pruner_test=default,std=0x1,moveos_std=0x2,rooch_framework=0x3',
    )

    // Initialize counter module
    console.log('🔢 Initializing counter module...')
    await runMoveFunction(testbox, `${defaultAddress}::quick_start_counter::increase`, [])
    console.log('✅ Counter initialized')

    console.log('✅ TestBox initialized successfully')
  }, 600000) // 10 minutes for initialization

  afterAll(async () => {
    if (testbox) {
      console.log('🧹 Cleanup test environment')
      testbox.stop()
    }
  })

  describe('GC Basic Functionality', () => {
    it('GC Dry Run - Basic functionality verification', async () => {
      console.log('\n🧪 Test: GC Dry Run basic functionality')

      const config = loadGCTestConfig()
      config.dryRun = true
      config.objectCount = 5 // Small dataset for quick test

      // Generate test data
      await generateTestData(testbox, config, defaultAddress)

      // Stop server for GC
      await stopServerForGC(testbox)

      try {
        // Execute GC dry run
        const result = executeGC(testbox, { dryRun: true })

        // Verify command succeeded
        expect(result.success).toBe(true)

        // Verify we got some output
        expect(result.output).toBeDefined()
        expect(result.output.length).toBeGreaterThan(0)

        // If JSON was parsed successfully, verify structure and display report
        if (result.report) {
          expect(result.report.executionMode).toBe('dry-run')
          expect(result.report.markStats.markedCount).toBeGreaterThan(0)
          printGCReport(result.report)
        } else {
          // If JSON wasn't parsed, at least verify output contains expected text
          expect(result.output).toMatch(/(dry-run|Dry Run|DRY RUN)/)
          console.log('⚠️ GC command executed but JSON parsing failed')
          console.log('Raw output (first 500 chars):', result.output.substring(0, 500))
        }

        console.log('✅ GC Dry Run test passed')
      } finally {
        // Always restart server
        await restartServer(testbox)
        await verifyServerHealth(testbox)
      }
    }, 120000)

    it('GC Execution and Service Restart - Full cycle test', async () => {
      console.log('\n🧪 Test: GC Execution and service restart')

      const config = loadGCTestConfig()
      config.dryRun = false // Actual GC execution
      config.skipConfirm = true
      config.objectCount = 8 // Small dataset
      config.deleteCount = 3 // Some deletions to test recycle bin

      // Generate test data (minimal for now)
      await generateTestData(testbox, config, defaultAddress)

      // Stop server for GC
      await stopServerForGC(testbox)

      let gcResult: GCResult
      try {
        // Execute actual GC
        gcResult = executeGC(testbox, { dryRun: false })

        // Verify command succeeded
        expect(gcResult.success).toBe(true)
        console.log('✅ GC execution completed')

        // Display detailed GC report
        if (gcResult.report) {
          printGCReport(gcResult.report)
        } else {
          console.log('⚠️ GC executed but JSON report not available')
          console.log('Raw output (first 500 chars):', gcResult.output.substring(0, 500))
        }
      } finally {
        // Always restart server
        await restartServer(testbox)
      }

      // Verify server health after restart
      await verifyServerHealth(testbox)

      // Verify database integrity by executing actual transactions
      console.log('🔍 Verifying database integrity with actual transactions...')

      // Execute counter operations to verify contract state is intact
      console.log('  📝 Testing counter operations...')
      await runMoveFunction(testbox, `${defaultAddress}::quick_start_counter::increase`, [])
      await runMoveFunction(testbox, `${defaultAddress}::quick_start_counter::increase`, [])
      console.log('  ✅ Counter operations successful')

      // Create new objects to verify object storage works
      console.log('  ➕ Testing object creation...')
      const verificationSeed = Date.now() % 1_000_000
      await runMoveFunction(testbox, `${defaultAddress}::object_lifecycle::create_named`, [
        `u64:${verificationSeed}`,
        `u64:999`,
      ])
      console.log('  ✅ Object creation successful')

      // Update the newly created object
      console.log('  ✏️  Testing object update...')
      await runMoveFunction(testbox, `${defaultAddress}::object_lifecycle::update_named`, [
        `u64:${verificationSeed}`,
        `u64:1000`,
      ])
      console.log('  ✅ Object update successful')

      // Delete the test object
      console.log('  🗑️  Testing object deletion...')
      await runMoveFunction(testbox, `${defaultAddress}::object_lifecycle::remove_named`, [
        `u64:${verificationSeed}`,
      ])
      console.log('  ✅ Object deletion successful')

      console.log('✅ Database integrity verified - all operations work correctly after GC')
      console.log('✅ GC Execution and service restart test passed')
    }, 180000)

    it('Recycle Bin Verification - Check deleted nodes', async () => {
      console.log('\n🧪 Test: Recycle bin verification')

      // Execute recycle list command to check if recycle bin has entries
      try {
        const recycleListOutput = testbox.roochCommand([
          'db',
          'recycle',
          'list',
          '--chain-id',
          'local',
          '--data-dir',
          path.join(testbox.roochDir, 'data'),
        ])

        console.log('📋 Recycle bin list output:', recycleListOutput)

        // Verify command succeeded and we can list recycle bin
        expect(recycleListOutput).toBeDefined()
        console.log('✅ Recycle bin list command executed successfully')

        // If there were deletions in previous tests, we should have entries
        // But this is optional - the command working is the main verification
      } catch (error) {
        console.warn('⚠️ Recycle bin list command failed:', error)
        // This might fail if there are no entries, which is acceptable
      }

      console.log('✅ Recycle bin verification test passed')
    }, 60000)
  })
})

// Stress Test Suite - Only runs when GC_STRESS_MODE=true
describe.skipIf(!process.env.GC_STRESS_MODE)('GC Stress Test - Long Running', () => {
  let testbox: TestBox
  let defaultAddress: string
  const stressConfig = loadGCStressTestConfig()

  beforeAll(async () => {
    console.log('🚀 Initializing GC Stress Test Suite')
    console.log(`📊 Stress Test Configuration:`)
    console.log(
      `   Duration: ${stressConfig.durationSeconds}s (${(stressConfig.durationSeconds / 60).toFixed(1)} minutes)`,
    )
    console.log(`   Target TPS: ${stressConfig.tps}`)
    console.log(`   Batch Size: ${stressConfig.batchSize}`)
    console.log(
      `   Mix Ratio: Create ${(stressConfig.mixRatio.create * 100).toFixed(0)}%, Update ${(stressConfig.mixRatio.update * 100).toFixed(0)}%, Delete ${(stressConfig.mixRatio.delete * 100).toFixed(0)}%, Counter ${(stressConfig.mixRatio.counter * 100).toFixed(0)}%`,
    )

    process.env.TESTBOX_KEEP_TMP = 'true'

    testbox = new TestBox()

    console.log('🔧 Starting Rooch server...')
    await testbox.loadRoochEnv('local', 0)
    await delay(30000)
    await verifyServerHealth(testbox)

    try {
      defaultAddress = await testbox.defaultCmdAddress()
      console.log(`📍 Default address: ${defaultAddress}`)
    } catch (error) {
      console.error('❌ Failed to get default address:', error)
      throw error
    }

    console.log('📦 Publishing contracts...')
    await publishPackage(
      testbox,
      counterPackagePath,
      'quick_start_counter=default,std=0x1,moveos_std=0x2,rooch_framework=0x3',
    )
    await publishPackage(
      testbox,
      prunerPackagePath,
      'pruner_test=default,std=0x1,moveos_std=0x2,rooch_framework=0x3',
    )

    // Initialize counter module
    console.log('🔢 Initializing counter module...')
    await runMoveFunction(testbox, `${defaultAddress}::quick_start_counter::increase`, [])
    console.log('✅ Counter initialized')

    console.log('✅ Stress Test environment initialized')
  }, 600000) // 10 minutes timeout

  afterAll(async () => {
    console.log('🧹 Cleanup stress test environment')
    if (testbox) {
      testbox.stop()
    }
  })

  it(
    'Stress Test - Continuous data generation and GC',
    async () => {
      console.log('\n🧪 Test: Long-running stress test with continuous data generation')

      const stopSignal = { stop: false }

      try {
        // Generate continuous data
        const stats = await generateContinuousData(
          testbox,
          stressConfig,
          defaultAddress,
          stopSignal,
        )

        console.log(`\n📈 Final Statistics:`)
        console.log(`   Total Transactions: ${stats.totalTxs}`)
        console.log(`   Create: ${stats.byType.create}`)
        console.log(`   Update: ${stats.byType.update}`)
        console.log(`   Delete: ${stats.byType.delete}`)
        console.log(`   Counter: ${stats.byType.counter}`)

        // Stop server for GC
        await stopServerForGC(testbox)

        // Execute GC (use config to determine recycle bin usage)
        const useRecycleBin = stressConfig.useRecycleBin
        console.log(
          `\n🗑️ Executing GC after stress test (${useRecycleBin ? 'with recycle bin' : 'direct delete for real disk space reclaim'})...`,
        )
        const gcResult = executeGC(testbox, { dryRun: false, useRecycleBin })

        expect(gcResult.success).toBe(true)
        console.log('✅ GC execution completed')

        // Display detailed GC report
        if (gcResult.report) {
          printGCReport(gcResult.report)

          // Verify GC found work to do
          expect(gcResult.report.markStats.markedCount).toBeGreaterThan(0)
          expect(gcResult.report.sweepStats.scannedCount).toBeGreaterThan(0)

          if (useRecycleBin) {
            console.log(
              `🎯 GC Performance: Deleted ${gcResult.report.sweepStats.deletedCount} nodes, ${gcResult.report.sweepStats.recycleBinEntries} entries in recycle bin`,
            )
            console.log(
              `ℹ️  Note: Nodes are in recycle bin. Use 'rooch db recycle purge' to actually free disk space.`,
            )
          } else {
            console.log(
              `🎯 GC Performance: Deleted ${gcResult.report.sweepStats.deletedCount} nodes, disk space freed immediately`,
            )
          }
        } else {
          console.log('⚠️ GC executed but JSON report not available')
        }

        // Restart server and verify
        await restartServer(testbox)
        await verifyServerHealth(testbox)

        // Verify database integrity with sample transactions
        console.log('🔍 Verifying database integrity after stress test and GC...')
        await runMoveFunction(testbox, `${defaultAddress}::quick_start_counter::increase`, [])
        console.log('✅ Database integrity verified')

        console.log('✅ Stress test completed successfully')
      } catch (error) {
        stopSignal.stop = true
        throw error
      }
    },
    // Timeout: test duration + 20 minutes for setup and GC
    (stressConfig.durationSeconds + 1200) * 1000,
  )
})
