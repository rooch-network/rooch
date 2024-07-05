// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0
import { Wallet } from './wallet.js'

export interface WalletsEventsListeners {
  /**
   * Emitted when Wallets are registered.
   *
   * @param wallets Wallets that were registered.
   */
  register(...wallets: Wallet[]): void

  /**
   * Emitted when Wallets are unregistered.
   *
   * @param wallets Wallets that were unregistered.
   */
  unregister(...wallets: Wallet[]): void
}

/**
 * Names of {@link WalletsEventsListeners} that can be listened for.
 *
 * @group App
 */
export type WalletsEventNames = keyof WalletsEventsListeners
