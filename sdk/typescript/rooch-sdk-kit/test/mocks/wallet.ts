// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

export abstract class Wallet {
  /**
   * Connects the wallet.
   * @returns A promise that resolves to an array of wallet accounts.
   */
  abstract connect(): Promise<any[]>

  /**
   * Signs a message.
   * @param msg - The message to sign.
   * @returns A promise that resolves to the signature string.
   */
  abstract sign(msg: any): Promise<any>

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
}
