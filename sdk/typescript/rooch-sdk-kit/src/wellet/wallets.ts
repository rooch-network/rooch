// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

import { Wallet } from './wallet.js'

let wallets: Wallets | undefined = undefined
const registered = new Set<Wallet>()
const listeners: { [E in WalletsEventNames]?: WalletsEventsListeners[E][] } = {}

export function getWallets(): Wallets {
  if (wallets) return wallets
  wallets = Object.freeze({ register, get, on })

  return wallets
}

export interface Wallets {
  /**
   * Get all Wallets that have been registered.
   *
   * @return Registered Wallets.
   */
  get(): readonly Wallet[]

  /**
   * Add an event listener and subscribe to events for Wallets that are
   * {@link WalletsEventsListeners.register | registered} and
   * {@link WalletsEventsListeners.unregister | unregistered}.
   *
   * @param event    Event type to listen for. {@link WalletsEventsListeners.register | `register`} and
   * {@link WalletsEventsListeners.unregister | `unregister`} are the only event types.
   * @param listener Function that will be called when an event of the type is emitted.
   *
   * @return
   * `off` function which may be called to remove the event listener and unsubscribe from events.
   *
   * As with all event listeners, be careful to avoid memory leaks.
   */
  on<E extends WalletsEventNames>(event: E, listener: WalletsEventsListeners[E]): () => void

  /**
   * Register Wallets. This can be used to programmatically wrap non-standard wallets as Standard Wallets.
   *
   * Apps generally do not need to, and should not, call this.
   *
   * @param wallets Wallets to register.
   *
   * @return
   * `unregister` function which may be called to programmatically unregister the registered Wallets.
   *
   * Apps generally do not need to, and should not, call this.
   */
  register(...wallets: Wallet[]): () => void
}

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

export function register(...wallets: Wallet[]) {
  wallets.forEach((wallet) => registered.add(wallet))
  listeners['register']?.forEach((listener) => guard(() => listener(...wallets)))

  return function unregister(): void {
    wallets.forEach((wallet) => registered.delete(wallet))
    listeners['unregister']?.forEach((listener) => guard(() => listener(...wallets)))
  }
}

function get(): readonly Wallet[] {
  return [...registered]
}

function on<E extends WalletsEventNames>(
  event: E,
  listener: WalletsEventsListeners[E],
): () => void {
  listeners[event]?.push(listener) || (listeners[event] = [listener])
  // Return a function that removes the event listener.
  return function off(): void {
    listeners[event] = listeners[event]?.filter((existingListener) => listener !== existingListener)
  }
}

function guard(callback: () => void) {
  try {
    callback()
  } catch (error) {
    console.error(error)
  }
}
