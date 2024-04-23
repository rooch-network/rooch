// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

import { sha3_256 } from '@noble/hashes/sha3'
import { WalletAccount } from '../WalletAccount'
import {
  IAuthorizer,
  RoochClient,
  SerializedSignature,
  runtime,
  uint8Array2SeqNumber,
} from '@roochnetwork/rooch-sdk'
import { Buffer } from 'buffer'
import { SupportChain } from '../../feature'

export const RoochSignPrefix = 'Rooch tx hash:\n'

export abstract class BaseWallet implements IAuthorizer {
  client: RoochClient
  account: WalletAccount | undefined
  installed: boolean | undefined
  name: string | undefined
  onAccountsChangedWrapper?: (addresses: string[]) => void

  constructor(client: RoochClient) {
    this.client = client
  }

  /**
   * Connects the wallet.
   * @returns A promise that resolves to an array of wallet accounts.
   */
  abstract connect(): Promise<WalletAccount[]>

  /**
   * Signs a message.
   * @param msg - The message to sign.
   * @returns A promise that resolves to the signature string.
   */
  abstract sign(msg: string): Promise<string>

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
  abstract onAccountsChanged(callback: (accounts: Array<WalletAccount>) => void): void

  /**
   * Removes a previously registered callback function for account changes.
   * @param callback - The callback function to be removed.
   */
  abstract removeAccountsChanged(callback: (accounts: Array<WalletAccount>) => void): void

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

  /**
   * Retrieves the scheme of the wallet. rooch validator definition
   * @returns The scheme of the wallet as a number.
   */
  abstract getScheme(): number

  /**
   * Check whether the wallet supports a chain
   */
  abstract getChain(): SupportChain

  /**
   * Normalizes the recovery ID.
   * @param recoveryID - The recovery ID to be normalized.
   * @returns The normalized recovery ID as a number.
   */
  protected abstract normalize_recovery_id(recoveryID: number): number

  /**
   * Converts the message, signature, and signature info to a serialized signature.
   * @param msg - The message in hexadecimal format.
   * @param signature - The signature string.
   * @param signatureInfo - Additional information about the signature.
   * @returns The serialized signature object.
   */
  protected abstract toSerializedSignature(
    msg: string,
    signature: string,
    signatureInfo: string,
  ): SerializedSignature

  /**
   * Signs a message.
   * @param msg - The message to sign as a Uint8Array.
   * @param msgInfo - Additional information about the message.
   * @returns A promise that resolves to the serialized signature object.
   */
  async signMessage(msg: Uint8Array, msgInfo?: any): Promise<SerializedSignature> {
    const digest = sha3_256(msg)
    return await this.signMessageWithHashed(digest, msgInfo)
  }

  /**
   * Signs a hashed message.
   * @param msgHash - The hashed message to sign as a Uint8Array.
   * @param msgInfo - Additional information about the message.
   * @returns A promise that resolves to the serialized signature object.
   */
  async signMessageWithHashed(msgHash: Uint8Array, msgInfo: any): Promise<SerializedSignature> {
    let msgHex = Buffer.from(msgHash).toString('hex')

    if (msgInfo.charAt(msgInfo.length - 1) !== '\n') {
      msgInfo += '\n'
    }

    msgInfo = msgInfo + RoochSignPrefix
    let fullMsg = msgInfo + msgHex

    const sign = await this.sign(fullMsg)

    return this.toSerializedSignature(msgHex, sign, msgInfo)
  }

  async auth(payload: Uint8Array, authInfo?: string): Promise<runtime.Authenticator> {
    return new runtime.Authenticator(
      BigInt(this.getScheme()),
      uint8Array2SeqNumber(await this.signMessage(payload, authInfo)),
    )
  }

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
