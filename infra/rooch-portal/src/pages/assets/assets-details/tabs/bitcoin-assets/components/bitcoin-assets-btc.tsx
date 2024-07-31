// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

import { useCurrentWallet } from '@roochnetwork/rooch-sdk-kit'
import { UtxoView } from '@/view/utxo-view.tsx'

export const BitcoinAssetsBtc = () => {
  const { wallet } = useCurrentWallet()

  return <UtxoView owner={wallet?.getBitcoinAddress().toStr() || ''} />
}
