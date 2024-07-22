#!/bin/env node

import { Command } from 'commander'

/////////////////////////////////////////////////////////////////////////////////////////////
// Start of Command Line Options Definitions
/////////////////////////////////////////////////////////////////////////////////////////////

const program = new Command()

program
  .name('BitSeed CLI Utility')
  .description('Command line utility for interacting with BitSeed')
  .version(require('../package.json').version)

program
  .command('server-version')
  .description('Get electrumx server version info')
  .action(async (options) => {
    try {
      console.log('options', options)
    } catch (error) {
      console.log(error)
    }
  })

program.parse()
