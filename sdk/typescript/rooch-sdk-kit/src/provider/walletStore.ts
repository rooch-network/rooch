// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

import { createStore } from 'zustand'
import { createJSONStorage, persist } from 'zustand/middleware'
import { StateStorage } from 'zustand/middleware'
import { ThirdPartyAddress } from '@roochnetwork/rooch-sdk'

import { SupportChain } from '../feature/index.js'

import { Wallet } from '../wellet/index.js'

type WalletConnectionStatus = 'disconnected' | 'connecting' | 'connected'

export type WalletActions = {
  setChain: (chain: SupportChain) => void
  setAddressSwitched: (selectedAccount: ThirdPartyAddress) => void
  setConnectionStatus: (connectionStatus: WalletConnectionStatus) => void
  setWalletConnected: (
    wallet: Wallet,
    connectedAddress: readonly ThirdPartyAddress[],
    selectedAddress: ThirdPartyAddress | null,
  ) => void
  updateWalletAddresses: (accounts: readonly ThirdPartyAddress[]) => void
  setWalletDisconnected: () => void
}

export type WalletStore = ReturnType<typeof createWalletStore>

export type WalletStoreState = {
  autoConnectEnabled: boolean
  currentChain: SupportChain
  currentWallet: Wallet | undefined
  wallets: Wallet[]
  addresses: readonly ThirdPartyAddress[]
  currentAddress: ThirdPartyAddress | undefined
  lastConnectedAddress: string | undefined
  lastConnectedWalletName: string | undefined
  connectionStatus: WalletConnectionStatus
} & WalletActions

type WalletConfiguration = {
  autoConnectEnabled: boolean
  chain: SupportChain
  currentWallet: Wallet | undefined
  wallets: Wallet[]
  storage: StateStorage
  storageKey: string
}

export function createWalletStore({
  chain,
  currentWallet,
  wallets,
  storage,
  storageKey,
  autoConnectEnabled,
}: WalletConfiguration) {
  return createStore<WalletStoreState>()(
    persist(
      (set, get) => ({
        currentChain: chain,
        autoConnectEnabled,
        currentWallet,
        wallets,
        addresses: [],
        currentAddress: undefined,
        lastConnectedAddress: undefined,
        lastConnectedWalletName: undefined,
        connectionStatus: 'disconnected',
        setChain(chain) {
          const currentChain = get().currentChain

          if (currentChain === chain) {
            return
          }
          set(() => ({
            currentChain: chain,
            accounts: [],
            // currentWallet: supportWallets.find((v) => v.getSupportNetworks()),
            sessionAccount: null,
            connectionStatus: 'disconnected',
          }))
        },
        setConnectionStatus(connectionStatus) {
          set(() => ({
            connectionStatus,
          }))
        },
        setWalletConnected(wallet, connectedAddresses, selectedAddress) {
          set(() => ({
            currentWallet: wallet,
            accounts: connectedAddresses,
            currentAddress: selectedAddress || undefined,
            lastConnectedWalletName: wallet.getName(),
            lastConnectedAddress: selectedAddress?.toStr(),
            connectionStatus: 'connected',
          }))
        },
        setWalletDisconnected() {
          set(() => ({
            accounts: [],
            currentAddress: undefined,
            lastConnectedWalletName: undefined,
            lastConnectedAddress: undefined,
            connectionStatus: 'disconnected',
          }))
        },
        setAddressSwitched(selected) {
          set(() => ({
            currentAddress: selected,
            lastConnectedAddress: selected.toStr() ?? '',
          }))
        },
        updateWalletAddresses(addresses) {
          const currentAddr = get().currentAddress
          set(() => ({
            currentAddress:
              (currentAddr && addresses.find((addr) => addr.toStr() === currentAddr.toStr())) ||
              addresses[0],
          }))
        },
      }),
      {
        name: storageKey,
        storage: createJSONStorage(() => storage),
        partialize: ({ lastConnectedWalletName, lastConnectedAddress }) => ({
          lastConnectedWalletName,
          lastConnectedAddress,
        }),
      },
    ),
  )
}
