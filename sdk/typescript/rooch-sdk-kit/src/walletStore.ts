// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

import { createStore } from 'zustand'
import { createJSONStorage, persist } from 'zustand/middleware'
import { StateStorage } from 'zustand/middleware'
import { IAccount } from '@roochnetwork/rooch-sdk'

import { WalletAccount } from './types/WalletAccount'
import { BaseWallet } from './types/wellet/baseWallet'

type WalletConnectionStatus = 'disconnected' | 'connecting' | 'connected'
type SessionKeyStatus = 'valid' | 'invalid' | 'creating'

export type WalletActions = {
  setAccountSwitched: (selectedAccount: WalletAccount) => void
  setConnectionStatus: (connectionStatus: WalletConnectionStatus) => void
  setWalletConnected: (
    connectedAccounts: readonly WalletAccount[],
    selectedAccount: WalletAccount | null,
  ) => void
  updateWalletAccounts: (accounts: readonly WalletAccount[]) => void
  setWalletDisconnected: () => void
  setSessionAccountStatus: (status: SessionKeyStatus) => void
  setSessionAccount: (session: IAccount) => void
}

export type WalletStore = ReturnType<typeof createWalletStore>

export type StoreState = {
  autoConnectEnabled: boolean
  // TODO: Support multiple accounts ?
  sessionAccount: IAccount | null
  sessionAccountStatus: SessionKeyStatus
  wallet: BaseWallet
  roochAddress: string
  accounts: readonly WalletAccount[]
  currentAccount: WalletAccount | null
  lastConnectedAccountAddress: string | null
  lastConnectedWalletName: string | null
  connectionStatus: WalletConnectionStatus
} & WalletActions

type WalletConfiguration = {
  autoConnectEnabled: boolean
  wallet: BaseWallet
  storage: StateStorage
  storageKey: string
}

export function createWalletStore({
  wallet,
  storage,
  storageKey,
  autoConnectEnabled,
}: WalletConfiguration) {
  return createStore<StoreState>()(
    persist(
      (set, get) => ({
        autoConnectEnabled,
        sessionAccount: null,
        sessionAccountStatus: 'invalid',
        roochAddress: '',
        wallet,
        accounts: [],
        currentWallet: null,
        currentAccount: null,
        lastConnectedAccountAddress: null,
        lastConnectedWalletName: null,
        connectionStatus: 'disconnected',
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
            currentWallet: null,
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
        setSessionAccountStatus(sessionAccountStatus) {
          set(() => ({
            sessionAccountStatus,
          }))
        },
      }),
      {
        name: storageKey,
        storage: createJSONStorage(() => storage),
        partialize: ({ lastConnectedWalletName, lastConnectedAccountAddress }) => ({
          lastConnectedWalletName,
          lastConnectedAccountAddress,
        }),
      },
    ),
  )
}
