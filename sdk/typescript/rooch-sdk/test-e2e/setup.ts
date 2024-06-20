// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0
import { execSync } from 'child_process'

import { RoochAddress } from '@/address'
import { getRoochNodeUrl, RoochClient } from '@/client'
import { Secp256k1Keypair } from '@/keypairs'
import { Transaction } from '@/transactions'

export const DEFAULT_NODE_URL = import.meta.env.VITE_FULLNODE_URL ?? getRoochNodeUrl('localnet')

let _defaultCmdAddress = ''

export class TestBox {
  keypair: Secp256k1Keypair
  client: RoochClient

  constructor(keypair: Secp256k1Keypair, client: RoochClient) {
    this.keypair = keypair
    this.client = client
  }

  address(): RoochAddress {
    return this.keypair.getRoochAddress()
  }

  delay(ms: number) {
    return new Promise((resolve) => setTimeout(resolve, ms))
  }

  async signAndExecuteTransaction(tx: Transaction) {
    const result = await this.client.signAndExecuteTransaction({
      transaction: tx,
      signer: this.keypair,
    })

    return (result.execution_info.status.type = 'executed')
  }
}

export function getClient(url = DEFAULT_NODE_URL): RoochClient {
  return new RoochClient({
    url,
  })
}

export async function setup(opts?: { url?: string }): Promise<TestBox> {
  const kp = Secp256k1Keypair.generate()
  const client = getClient(opts?.url)

  return new TestBox(kp, client)
}

export async function cmd(args: string[] | string): Promise<string> {
  return execSync(`cargo run --bin rooch ${typeof args === 'string' ? args : args.join(' ')}`, {
    encoding: 'utf-8',
    stdio: 'pipe',
  })
}

export async function cmdPublishPackage(
  packagePath: string,
  options: {
    namedAddresses: string
  } = {
    namedAddresses: 'default',
  },
) {
  const result = await cmd(
    `move publish -p ${packagePath} --named-addresses rooch_examples=${options.namedAddresses}`,
  )
  const { execution_info } = JSON.parse(result)

  return execution_info?.status?.type === 'executed'
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
export async function defaultCmdAddress(): Promise<string> {
  if (!_defaultCmdAddress) {
    const accounts = JSON.parse(await cmd(['account', 'list', '--json']))

    for (const account of accounts) {
      if (account.active) {
        _defaultCmdAddress = account.local_account.hex_address
      }
    }

    if (!_defaultCmdAddress) {
      throw new Error('No active account address')
    }
  }

  return _defaultCmdAddress
}
