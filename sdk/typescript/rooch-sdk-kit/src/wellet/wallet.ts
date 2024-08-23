// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

import { ThirdPartyAddress, Bytes, Signer } from '@roochnetwork/rooch-sdk'

import { SupportChain } from '../feature/index.js'

export abstract class Wallet extends Signer {
  protected address: ThirdPartyAddress[] | undefined
  protected publicKey: string | undefined
  protected currentAddress: ThirdPartyAddress | undefined

  /**
   * Connects the wallet.
   * @returns A promise that resolves to an array of wallet accounts.
   */
  abstract connect(): Promise<ThirdPartyAddress[]>

  abstract getName(): string

  abstract getIcon(theme?: 'dark' | 'light'): string

  abstract getInstallUrl(): string

  abstract getDescription(): string

  /**
   * Signs a message.
   * @param msg - The message to sign.
   * @returns A promise that resolves to the signature string.
   */
  abstract sign(msg: Bytes): Promise<Bytes>

  /**
   * Switches the network.
   */
  abstract switchNetwork(network: string): void

  /**
   * Switches the account.
   * Note: Wallets with Bitcoin chain are not currently supported.
   */
  abstract switchAccount(address: string): void

  /**
   * Retrieves the current network of the wallet.
   * @returns The current network as a string.
   */
  abstract getNetwork(): string

  /**
   * Retrieves the supported networks of the wallet.
   * @returns An array of supported network strings.
   */
  abstract getSupportNetworks(): string[]

  /**
   * Registers a callback function to be invoked when accounts are changed.
   * @param callback - A function to be called when accounts are changed. It receives an array of account strings as its argument.
   */
  abstract onAccountsChanged(callback: (accounts: Array<string>) => void): void

  /**
   * Removes a previously registered callback function for account changes.
   * @param callback - The callback function to be removed.
   */
  abstract removeAccountsChanged(callback: (accounts: Array<string>) => void): void

  /**
   * Registers a callback function to be invoked when the network is changed.
   * @param callback - A function to be called when the network is changed. It receives the new network as its argument.
   */
  abstract onNetworkChanged(callback: (network: string) => void): void

  /**
   * Removes a previously registered callback function for network changes.
   * @param callback - The callback function to be removed.
   */
  abstract removeNetworkChanged(callback: (network: string) => void): void

  /**
   * Retrieves the target of the wallet.
   * @returns The target of the wallet.
   */
  abstract getTarget(): any

  abstract getChain(): SupportChain

  /**
   * Normalizes the recovery ID.
   * @param recoveryID - The recovery ID to be normalized.
   * @returns The normalized recovery ID as a number.
   */
  protected abstract normalize_recovery_id(recoveryID: number): number

  /**
   * Checks if the wallet is installed.
   * @returns A promise that resolves to true if the wallet is installed, otherwise false.
   */
  async checkInstalled(): Promise<boolean> {
    for (let i = 1; i < 10 && !this.getTarget(); i += 1) {
      await new Promise((resolve) => setTimeout(resolve, 100 * i))
    }
    return Promise.resolve(this.getTarget() !== undefined)
  }
}
