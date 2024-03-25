// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

import { createStore } from 'zustand'
import { createJSONStorage, persist } from 'zustand/middleware'
import { StateStorage } from 'zustand/middleware'
import { IAccount } from '@roochnetwork/rooch-sdk'

import { BaseWallet, WalletAccount } from './types'
import { SupportChain } from './feature'

type WalletConnectionStatus = 'disconnected' | 'connecting' | 'connected'

export type WalletActions = {
  setChain: (chain: SupportChain) => void
  setAccountSwitched: (selectedAccount: WalletAccount) => void
  setConnectionStatus: (connectionStatus: WalletConnectionStatus) => void
  setWalletConnected: (
    connectedAccounts: readonly WalletAccount[],
    selectedAccount: WalletAccount | null,
  ) => void
  updateWalletAccounts: (accounts: readonly WalletAccount[]) => void
  setWalletDisconnected: () => void
  setSessionAccount: (session: IAccount) => void
}

export type WalletStore = ReturnType<typeof createWalletStore>

export type StoreState = {
  autoConnectEnabled: boolean
  currentChain: SupportChain
  sessionAccount: IAccount | null
  currentWallet: BaseWallet
  installedWallets: BaseWallet[]
  accounts: readonly WalletAccount[]
  currentAccount: WalletAccount | null
  lastConnectedAccountAddress: string | null
  lastConnectedWalletName: string | null
  connectionStatus: WalletConnectionStatus
} & WalletActions

type WalletConfiguration = {
  autoConnectEnabled: boolean
  chain: SupportChain
  currentWallet: BaseWallet
  installedWallets: BaseWallet[]
  storage: StateStorage
  storageKey: string
}

export function createWalletStore({
  chain,
  currentWallet,
  installedWallets,
  storage,
  storageKey,
  autoConnectEnabled,
}: WalletConfiguration) {
  return createStore<StoreState>()(
    persist(
      (set, get) => ({
        currentChain: chain,
        autoConnectEnabled,
        sessionAccount: null,
        currentWallet,
        installedWallets,
        accounts: [],
        currentAccount: null,
        lastConnectedAccountAddress: null,
        lastConnectedWalletName: null,
        connectionStatus: 'disconnected',
        setChain(chain) {
          const currentChain = get().currentChain

          if (currentChain === chain) {
            return
          }
          set(() => ({
            currentChain: chain,
            accounts: [],
            // currentAccount: null,
            // currentWallet: installedWallets.find((v) => v.getSupportNetworks()),
            sessionAccount: null,
            connectionStatus: 'disconnected',
          }))
        },
        setConnectionStatus(connectionStatus) {
          set(() => ({
            connectionStatus,
          }))
        },
        setWalletConnected(connectedAccounts, selectedAccount) {
          set(() => ({
            accounts: connectedAccounts,
            currentAccount: selectedAccount,
            lastConnectedAccountAddress: selectedAccount?.getAddress(),
            connectionStatus: 'connected',
          }))
        },
        setWalletDisconnected() {
          set(() => ({
            accounts: [],
            currentAccount: null,
            lastConnectedWalletName: null,
            lastConnectedAccountAddress: null,
            connectionStatus: 'disconnected',
          }))
        },
        setAccountSwitched(selectedAccount) {
          set(() => ({
            currentAccount: selectedAccount,
            lastConnectedAccountAddress: selectedAccount.getAddress(),
          }))
        },
        updateWalletAccounts(accounts) {
          const currentAccount = get().currentAccount

          set(() => ({
            currentAccount:
              (currentAccount &&
                accounts.find((account) => account.getAddress() === currentAccount.getAddress())) ||
              accounts[0],
          }))
        },
        setSessionAccount(sessionAccount) {
          set(() => ({
            sessionAccount,
            sessionKeyStatus: 'valid',
          }))
        },
      }),
      {
        name: storageKey,
        storage: createJSONStorage(() => storage),
        partialize: ({
          lastConnectedWalletName,
          lastConnectedAccountAddress,
          currentAccount,
          sessionAccount,
        }) => ({
          lastConnectedWalletName,
          lastConnectedAccountAddress,
          currentAccount,
          sessionAccount,
        }),
      },
    ),
  )
}
