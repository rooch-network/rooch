// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

import { Command } from 'commander'
import { Generator } from './generator.mjs'

const main = async (opts) => {
  const generator = new Generator({
    openrpcDocument: '../../../crates/rooch-open-rpc-spec/schemas/openrpc.json',
    outDir: opts.outputDir || './src/generated/client',
  })

  try {
    await generator.execute()
    console.log('generate rooch typescript client ok!')
  } catch (e) {
    console.error('generate rooch typescript client error:', e)
  }
}

const program = new Command()
program
  .option(
    '-o, --output-dir <string>',
    'Output dir for generated typescript code.',
  )
  .parse()

main(program.opts())
