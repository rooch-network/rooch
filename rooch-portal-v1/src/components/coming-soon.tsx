// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0
import { BitcoinIcon } from 'lucide-react'

export const ComingSoon = () => (
  <div className="flex flex-col items-center justify-center text-center text-xl text-muted-foreground my-10 animate-pulse">
    <BitcoinIcon className="w-12 h-12 mb-4 text-orange-500" />
    <p className="mb-2 font-semibold">Coming Soon...</p>
    <p className="text-base text-gray-500">
      We're working hard to bring this feature to you. Stay tuned!
    </p>
  </div>
)
