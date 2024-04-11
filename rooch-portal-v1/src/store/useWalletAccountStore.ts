import { create } from 'zustand'
import { WalletAccount } from '@roochnetwork/rooch-sdk-kit'

interface WalletAccountState {
  walletAccount: WalletAccount | null
  roochAddress: string | null
  setWalletAccount: (account: WalletAccount | null) => void
  setRoochAddress: (address: string | null) => void
}

export const useWalletAccountStore = create<WalletAccountState>((set) => ({
  walletAccount: null,
  roochAddress: null,
  setWalletAccount: (account) => set({ walletAccount: account }),
  setRoochAddress: (address) => set({ roochAddress: address }),
}))
