// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

'use client'

import { RoochProvider } from '@/context/rooch'
import { ETHProvider } from '@/context/wallet'
import Counter from '@/pages/counter'

export default function Home() {
  return (
    <RoochProvider>
      <ETHProvider>
        <Counter />
      </ETHProvider>
    </RoochProvider>
  )
}
