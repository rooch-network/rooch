// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0
import { AlertCircle } from 'lucide-react'

export const ComingSoon = () => (
  <div className="flex flex-col items-center justify-center text-center text-xl text-muted-foreground mt-10 animate-pulse">
    <AlertCircle className="w-12 h-12 mb-4 text-blue-500" />
    <p className="mb-2 font-semibold">Coming Soon!</p>
    <p className="text-base text-gray-500">
      We're working hard to bring this feature to you. Stay tuned!
    </p>
  </div>
)
