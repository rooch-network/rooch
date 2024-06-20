// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

import { createStore } from 'zustand'
import { createJSONStorage, persist } from 'zustand/middleware'
import { StateStorage } from 'zustand/middleware'

import { SupportChain } from '@/feature'
import { BitcoinAddress } from '@roochnetwork/rooch-sdk'
import { Wallet } from '@/wellet'

type WalletConnectionStatus = 'disconnected' | 'connecting' | 'connected'

export type WalletActions = {
  setChain: (chain: SupportChain) => void
  setAddressSwitched: (selectedAccount: BitcoinAddress) => void
  setConnectionStatus: (connectionStatus: WalletConnectionStatus) => void
  setWalletConnected: (
    wallet: Wallet,
    connectedAddress: readonly BitcoinAddress[],
    selectedAddress: BitcoinAddress | null,
  ) => void
  updateWalletAddresses: (accounts: readonly BitcoinAddress[]) => void
  setWalletDisconnected: () => void
}

export type WalletStore = ReturnType<typeof createWalletStore>

export type WalletStoreState = {
  autoConnectEnabled: boolean
  currentChain: SupportChain
  currentWallet: Wallet | undefined
  wallets: Wallet[]
  addresses: readonly BitcoinAddress[]
  currentAddress: BitcoinAddress | undefined
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
        setWalletConnected(wallet, connectedAccounts, selectedAccount) {
          set(() => ({
            currentWallet: wallet,
            accounts: connectedAccounts,
            currentAccount: selectedAccount,
            lastConnectedWalletName: wallet.getName(),
            lastConnectedAccountAddress: selectedAccount?.toStr(),
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
