// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0
import { Wallet } from 'lucide-react'

export const ConnectWalletHint = () => (
  <div className="flex flex-col items-center justify-center text-center p-40">
    <Wallet className="w-12 h-12 mb-4 text-zinc-500" />
    <p className="text-xl text-zinc-500 font-semibold">Haven't connected to wallet</p>
    <p className="text-sm text-muted-foreground mt-2">
      Please connect your wallet to view your assets.
    </p>
  </div>
)



